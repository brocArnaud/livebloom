use anyhow::{Context, Result};
use libloading::Library;
use nix::sys::memfd::{memfd_create, MemFdCreateFlag};
use nix::unistd::write;
use std::collections::HashMap;
use std::ffi::CStr;
use std::fs;
use std::os::fd::{AsRawFd, OwnedFd};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;
use tracing::{error, info, warn};

#[derive(Default)]
pub struct AppState {
    pub counter: u64,
}

#[derive(Default, Clone, Debug)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub dependencies: HashMap<String, String>,
}

impl Manifest {
    /// Generate a Cargo.toml string for the cdylib crate.
    pub fn to_cargo_toml(&self) -> String {
        let mut toml = format!(
            "[package]\nname = \"{}\"\nversion = \"{}\"\nedition = \"2021\"\n\n\
             [lib]\nname = \"livebloom_core\"\npath = \"src/lib.rs\"\ncrate-type = [\"cdylib\"]\n\n\
             [dependencies]\n",
            self.name, self.version
        );
        for (k, v) in &self.dependencies {
            toml += &format!("{} = \"{}\"\n", k, v);
        }
        toml
    }
}

/// A loaded module: keeps the Library and its backing memfd alive together.
/// When dropped, both the Library (dlclose) and the FD are cleaned up.
struct LoadedModule {
    _fd: OwnedFd,
    _library: Arc<Library>,
}

#[derive(Clone)]
pub struct LiveBloom {
    sources: Arc<Mutex<HashMap<PathBuf, String>>>,
    manifest: Arc<Mutex<Manifest>>,
    routes: Arc<Mutex<HashMap<String, String>>>,
    loaded_modules: Arc<Mutex<HashMap<String, LoadedModule>>>,
    /// Holds an Arc<Library> so callers can use it without holding the modules lock.
    current_core: Arc<Mutex<Option<Arc<Library>>>>,
    /// Last compilation error, shown to users via HTTP.
    last_error: Arc<Mutex<Option<String>>>,
    /// Shared cargo home directory for caching dependencies across rebuilds.
    cargo_home: Arc<PathBuf>,
    pub state: Arc<Mutex<AppState>>,
}

fn lock_or_default<T: Default>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| {
        warn!("Mutex was poisoned, recovering");
        poisoned.into_inner()
    })
}

impl LiveBloom {
    pub fn new(project_name: &str) -> Result<Self> {
        let mut sources = HashMap::new();

        sources.insert(
            PathBuf::from("src/content.rs"),
            r#"
#[export_name = "get_html"]
pub extern "C" fn get_html() -> *mut std::ffi::c_char {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let html = format!("<h1>🕒 Horloge live : {}</h1>", now);
    let c_string = std::ffi::CString::new(html).unwrap_or_default();
    c_string.into_raw()
}

#[export_name = "free_html"]
pub unsafe extern "C" fn free_html(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() {
        let _ = std::ffi::CString::from_raw(ptr);
    }
}
"#
            .to_string(),
        );

        let manifest = Manifest {
            name: project_name.to_string(),
            version: "0.1.0".into(),
            dependencies: HashMap::new(),
        };

        // Create a persistent cargo home for caching
        let cargo_home = std::env::temp_dir().join("livebloom_cargo_home");
        fs::create_dir_all(&cargo_home).context("Failed to create cargo home")?;

        Ok(Self {
            sources: Arc::new(Mutex::new(sources)),
            manifest: Arc::new(Mutex::new(manifest)),
            routes: Arc::new(Mutex::new(HashMap::new())),
            loaded_modules: Arc::new(Mutex::new(HashMap::new())),
            current_core: Arc::new(Mutex::new(None)),
            last_error: Arc::new(Mutex::new(None)),
            cargo_home: Arc::new(cargo_home),
            state: Arc::new(Mutex::new(AppState::default())),
        })
    }

    pub fn edit_file(&self, path: impl Into<PathBuf>, new_code: String) {
        let mut sources = lock_or_default(&self.sources);
        sources.insert(path.into(), new_code);
        info!("File modified in memory by agent");
    }

    pub fn add_dependency(&self, crate_name: String, version: String) {
        let mut m = lock_or_default(&self.manifest);
        m.dependencies.insert(crate_name.clone(), version);
        info!("Dependency '{}' added in RAM", crate_name);
    }

    pub fn add_route(&self, path: String, html: String) {
        let mut r = lock_or_default(&self.routes);
        info!("Route '{}' added dynamically by agent", path);
        r.insert(path, html);
    }

    pub fn get_route(&self, path: &str) -> Option<String> {
        let r = lock_or_default(&self.routes);
        r.get(path).cloned()
    }

    /// Generate the lib.rs content with mod declarations for all source files.
    fn generate_lib_rs(sources: &HashMap<PathBuf, String>) -> String {
        let mut lib_rs = String::from("#![allow(unused)]\n");
        for path in sources.keys() {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if stem != "lib" {
                    lib_rs += &format!("pub mod {};\n", stem);
                }
            }
        }
        lib_rs
    }

    pub fn rebuild_and_swap(&self, module_name: &str) -> Result<()> {
        let temp_dir = TempDir::new().context("Failed to create temp dir")?;
        let temp_path = temp_dir.path();

        // Write Cargo.toml
        let manifest = lock_or_default(&self.manifest);
        let cargo_toml = manifest.to_cargo_toml();
        drop(manifest);
        fs::write(temp_path.join("Cargo.toml"), cargo_toml)?;

        // Write source files
        fs::create_dir_all(temp_path.join("src"))?;
        let sources = lock_or_default(&self.sources);
        let lib_rs = Self::generate_lib_rs(&sources);
        fs::write(temp_path.join("src/lib.rs"), lib_rs)?;
        for (path, code) in &*sources {
            let file_path = temp_path.join(path);
            if let Some(p) = file_path.parent() {
                fs::create_dir_all(p)?;
            }
            fs::write(file_path, code)?;
        }
        drop(sources);

        // Build with shared cargo home
        let output = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .env("CARGO_HOME", self.cargo_home.as_ref())
            .current_dir(temp_path)
            .output()
            .context("Failed to run cargo")?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
            error!("Cargo build failed: {}", err_msg);
            let mut last_err = lock_or_default(&self.last_error);
            *last_err = Some(err_msg.clone());
            anyhow::bail!("cargo build failed: {}", err_msg);
        }

        // Clear last error on success
        {
            let mut last_err = lock_or_default(&self.last_error);
            *last_err = None;
        }

        let so_bytes =
            fs::read(temp_path.join("target/release/liblivebloom_core.so"))
                .context("Failed to read compiled .so")?;

        // Load into memfd
        let owned_fd: OwnedFd =
            memfd_create(c"livebloom_module", MemFdCreateFlag::MFD_CLOEXEC)
                .context("memfd_create failed")?;
        write(&owned_fd, &so_bytes).context("Failed to write to memfd")?;

        let lib_path = format!("/proc/self/fd/{}", owned_fd.as_raw_fd());
        let lib = unsafe { Library::new(&lib_path).context("Failed to load library from memfd")? };
        let lib_arc = Arc::new(lib);

        // Store module (old module + old FD dropped here)
        {
            let mut modules = lock_or_default(&self.loaded_modules);
            modules.insert(
                module_name.to_string(),
                LoadedModule {
                    _fd: owned_fd,
                    _library: Arc::clone(&lib_arc),
                },
            );
        }

        // Update current_core for lock-free access during get_html
        if module_name == "core" {
            let mut core = lock_or_default(&self.current_core);
            *core = Some(lib_arc);
        }

        info!("Module '{}' recompiled + swapped IN MEMORY", module_name);
        Ok(())
    }

    /// Async version of rebuild_and_swap — runs cargo build on a blocking thread.
    pub async fn rebuild_and_swap_async(&self, module_name: &str) -> Result<()> {
        let this = self.clone();
        let name = module_name.to_string();
        tokio::task::spawn_blocking(move || this.rebuild_and_swap(&name))
            .await
            .context("spawn_blocking panicked")?
    }

    /// Get HTML from the loaded core module.
    /// Uses Arc<Library> so the library stays alive even if a swap happens concurrently.
    pub fn get_html(&self) -> String {
        // Get an Arc clone — safe even if module is swapped during execution
        let lib_arc = {
            let core = lock_or_default(&self.current_core);
            match core.as_ref() {
                Some(arc) => Arc::clone(arc),
                None => return self.fallback_html(),
            }
        };

        unsafe {
            let get_fn = lib_arc
                .get::<unsafe extern "C" fn() -> *mut std::ffi::c_char>(b"get_html");
            let free_fn = lib_arc
                .get::<unsafe extern "C" fn(*mut std::ffi::c_char)>(b"free_html");

            match (get_fn, free_fn) {
                (Ok(get), Ok(free)) => {
                    let ptr = get();
                    if ptr.is_null() {
                        return self.fallback_html();
                    }
                    let c_str = CStr::from_ptr(ptr);
                    let result = c_str.to_string_lossy().into_owned();
                    free(ptr);
                    result
                }
                _ => {
                    error!("Failed to load get_html/free_html symbols from module");
                    self.fallback_html()
                }
            }
        }
    }

    fn fallback_html(&self) -> String {
        let last_err = lock_or_default(&self.last_error);
        match last_err.as_ref() {
            Some(err) => format!(
                "<div style='color:red;text-align:left;font-family:monospace;white-space:pre-wrap'>\
                 <h2>⚠ Compilation Error</h2>{}</div>",
                html_escape(err)
            ),
            None => "<h1>Loading...</h1>".to_string(),
        }
    }

    pub fn last_error(&self) -> Option<String> {
        let e = lock_or_default(&self.last_error);
        e.clone()
    }
}

/// Minimal HTML escaping for error display.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_to_cargo_toml() {
        let m = Manifest {
            name: "test_project".to_string(),
            version: "1.0.0".to_string(),
            dependencies: HashMap::from([("serde".to_string(), "1.0".to_string())]),
        };
        let toml = m.to_cargo_toml();
        assert!(toml.contains("name = \"test_project\""));
        assert!(toml.contains("version = \"1.0.0\""));
        assert!(toml.contains("crate-type = [\"cdylib\"]"));
        assert!(toml.contains("serde = \"1.0\""));
    }

    #[test]
    fn test_manifest_empty_deps() {
        let m = Manifest {
            name: "empty".to_string(),
            version: "0.1.0".to_string(),
            dependencies: HashMap::new(),
        };
        let toml = m.to_cargo_toml();
        assert!(toml.contains("[dependencies]"));
        // No dep lines after [dependencies]
        let after_deps = toml.split("[dependencies]\n").nth(1).unwrap_or("");
        assert!(after_deps.trim().is_empty());
    }

    #[test]
    fn test_new_creates_default_source() {
        let bloom = LiveBloom::new("test").unwrap();
        let sources = lock_or_default(&bloom.sources);
        assert!(sources.contains_key(&PathBuf::from("src/content.rs")));
        let code = sources.get(&PathBuf::from("src/content.rs")).unwrap();
        assert!(code.contains("get_html"));
        assert!(code.contains("free_html"));
    }

    #[test]
    fn test_edit_file() {
        let bloom = LiveBloom::new("test").unwrap();
        bloom.edit_file("src/hello.rs", "fn hello() {}".to_string());
        let sources = lock_or_default(&bloom.sources);
        assert_eq!(
            sources.get(&PathBuf::from("src/hello.rs")).unwrap(),
            "fn hello() {}"
        );
    }

    #[test]
    fn test_add_dependency() {
        let bloom = LiveBloom::new("test").unwrap();
        bloom.add_dependency("tokio".to_string(), "1.0".to_string());
        let m = lock_or_default(&bloom.manifest);
        assert_eq!(m.dependencies.get("tokio").unwrap(), "1.0");
    }

    #[test]
    fn test_add_and_get_route() {
        let bloom = LiveBloom::new("test").unwrap();
        assert!(bloom.get_route("/hello").is_none());
        bloom.add_route("/hello".to_string(), "<h1>Hi</h1>".to_string());
        assert_eq!(bloom.get_route("/hello").unwrap(), "<h1>Hi</h1>");
    }

    #[test]
    fn test_get_route_missing() {
        let bloom = LiveBloom::new("test").unwrap();
        assert!(bloom.get_route("/nonexistent").is_none());
    }

    #[test]
    fn test_generate_lib_rs() {
        let mut sources = HashMap::new();
        sources.insert(PathBuf::from("src/content.rs"), String::new());
        sources.insert(PathBuf::from("src/utils.rs"), String::new());
        let lib_rs = LiveBloom::generate_lib_rs(&sources);
        assert!(lib_rs.contains("pub mod content;"));
        assert!(lib_rs.contains("pub mod utils;"));
        assert!(!lib_rs.contains("pub mod lib;"));
    }

    #[test]
    fn test_generate_lib_rs_skips_lib() {
        let mut sources = HashMap::new();
        sources.insert(PathBuf::from("src/lib.rs"), String::new());
        let lib_rs = LiveBloom::generate_lib_rs(&sources);
        assert!(!lib_rs.contains("pub mod lib;"));
    }

    #[test]
    fn test_get_html_no_module_loaded() {
        let bloom = LiveBloom::new("test").unwrap();
        let html = bloom.get_html();
        assert!(html.contains("Loading"));
    }

    #[test]
    fn test_fallback_html_with_error() {
        let bloom = LiveBloom::new("test").unwrap();
        {
            let mut e = lock_or_default(&bloom.last_error);
            *e = Some("error: expected `;`".to_string());
        }
        let html = bloom.fallback_html();
        assert!(html.contains("Compilation Error"));
        assert!(html.contains("expected"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("\"hello\""), "&quot;hello&quot;");
    }

    #[test]
    fn test_last_error_initially_none() {
        let bloom = LiveBloom::new("test").unwrap();
        assert!(bloom.last_error().is_none());
    }

    #[test]
    fn test_clone_shares_state() {
        let bloom = LiveBloom::new("test").unwrap();
        let bloom2 = bloom.clone();
        bloom.add_route("/test".to_string(), "hello".to_string());
        assert_eq!(bloom2.get_route("/test").unwrap(), "hello");
    }

    #[test]
    fn test_rebuild_and_swap_compiles_and_loads() {
        // Integration test: actually compiles Rust code and loads it
        let bloom = LiveBloom::new("integration_test").unwrap();
        let result = bloom.rebuild_and_swap("core");
        assert!(result.is_ok(), "rebuild_and_swap failed: {:?}", result.err());

        let html = bloom.get_html();
        // The default content.rs renders a clock
        assert!(html.contains("Horloge live"));
    }

    #[test]
    fn test_rebuild_swap_then_edit_and_swap_again() {
        let bloom = LiveBloom::new("swap_test").unwrap();
        bloom.rebuild_and_swap("core").unwrap();

        // Edit to different content
        bloom.edit_file(
            "src/content.rs",
            r#"
#[export_name = "get_html"]
pub extern "C" fn get_html() -> *mut std::ffi::c_char {
    let html = "SWAPPED_CONTENT_MARKER";
    let c_string = std::ffi::CString::new(html).unwrap_or_default();
    c_string.into_raw()
}

#[export_name = "free_html"]
pub unsafe extern "C" fn free_html(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() {
        let _ = std::ffi::CString::from_raw(ptr);
    }
}
"#
            .to_string(),
        );
        bloom.rebuild_and_swap("core").unwrap();

        let html = bloom.get_html();
        assert!(html.contains("SWAPPED_CONTENT_MARKER"));
    }

    #[test]
    fn test_rebuild_with_compile_error_stores_error() {
        let bloom = LiveBloom::new("err_test").unwrap();
        bloom.edit_file(
            "src/content.rs",
            "THIS IS NOT VALID RUST CODE !!!".to_string(),
        );
        let result = bloom.rebuild_and_swap("core");
        assert!(result.is_err());
        assert!(bloom.last_error().is_some());
    }

    #[test]
    fn test_add_multiple_source_files() {
        let bloom = LiveBloom::new("multi_src").unwrap();
        bloom.edit_file("src/content.rs", r#"
pub use crate::helpers::helper_value;

#[export_name = "get_html"]
pub extern "C" fn get_html() -> *mut std::ffi::c_char {
    let html = format!("VALUE={}", helper_value());
    let c_string = std::ffi::CString::new(html).unwrap_or_default();
    c_string.into_raw()
}

#[export_name = "free_html"]
pub unsafe extern "C" fn free_html(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() { let _ = std::ffi::CString::from_raw(ptr); }
}
"#.to_string());
        bloom.edit_file("src/helpers.rs", r#"
pub fn helper_value() -> i32 { 42 }
"#.to_string());

        let result = bloom.rebuild_and_swap("core");
        assert!(result.is_ok(), "Multi-file build failed: {:?}", result.err());
        let html = bloom.get_html();
        assert!(html.contains("VALUE=42"));
    }
}
