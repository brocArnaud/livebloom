use anyhow::{Context, Result};
use libloading::Library;
use nix::sys::memfd::{memfd_create, MemFdCreateFlag};
use nix::unistd::write;
use std::collections::HashMap;
use std::ffi::CStr;
use std::fs;
use std::mem;
use std::os::fd::{AsRawFd, OwnedFd};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

#[derive(Default)]
pub struct AppState {
    pub counter: u64,
}

#[derive(Default, Clone)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub dependencies: HashMap<String, String>,
}

#[derive(Clone)]
pub struct LiveBloom {
    sources: Arc<Mutex<HashMap<PathBuf, String>>>,
    manifest: Arc<Mutex<Manifest>>,
    routes: Arc<Mutex<HashMap<String, String>>>,
    loaded_modules: Arc<Mutex<HashMap<String, Library>>>,
    pub state: Arc<Mutex<AppState>>,
}

impl LiveBloom {
    pub fn new(project_name: &str) -> Self {
        let mut sources = HashMap::new();

        sources.insert(PathBuf::from("src/content.rs"), r#"
#[export_name = "get_html"]
pub extern "C" fn get_html(mode: i32, html_out: *mut u8, len: *mut usize) {
    eprintln!("🌸 [IN-MEMORY] get_html called for mode {}", mode);
    let html = match mode {
        0 => { let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(); format!("<h1>🕒 Horloge live : {}</h1>", now) }
        1 => "<h1>🌸 COUCOU ! Le swap a marché sans coupure !</h1>".to_string(),
        _ => "<h1>Contenu par défaut</h1>".to_string()
    };
    let bytes = html.as_bytes();
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), html_out, bytes.len()); *len = bytes.len(); }
}
"#.to_string());

        let manifest = Manifest {
            name: project_name.to_string(),
            version: "0.4.0".into(),
            dependencies: HashMap::from([
                ("axum".to_string(), "0.7".to_string()),
                ("rand".to_string(), "0.8".to_string()),
            ]),
        };

        Self {
            sources: Arc::new(Mutex::new(sources)),
            manifest: Arc::new(Mutex::new(manifest)),
            routes: Arc::new(Mutex::new(HashMap::new())),
            loaded_modules: Arc::new(Mutex::new(HashMap::new())),
            state: Arc::new(Mutex::new(AppState::default())),
        }
    }

    pub fn edit_file(&self, path: impl Into<PathBuf>, new_code: String) {
        let mut sources = self.sources.lock().unwrap();
        sources.insert(path.into(), new_code);
        println!("🌸 Fichier modifié en mémoire par agent");
    }

    pub fn add_dependency(&self, crate_name: String, version: String) {
        let mut m = self.manifest.lock().unwrap();
        m.dependencies.insert(crate_name.clone(), version);
        println!("🌸 Dépendance '{}' ajoutée en RAM", crate_name);
    }

    pub fn add_route(&self, path: String, html: String) {
        let mut r = self.routes.lock().unwrap();
        r.insert(path.clone(), html);
        println!(
            "🌸 Route '{}' ajoutée dynamiquement en mémoire par agent !",
            path
        );
    }

    pub fn get_route(&self, path: &str) -> Option<String> {
        let r = self.routes.lock().unwrap();
        r.get(path).cloned()
    }

    pub fn rebuild_and_swap(&self, module_name: &str) -> Result<()> {
        let temp_dir = TempDir::new().context("temp dir")?;
        let temp_path = temp_dir.path();

        let mut cargo_toml = "[package]\nname = \"livebloom_core\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[lib]\nname = \"livebloom_core\"\npath = \"src/lib.rs\"\ncrate-type = [\"cdylib\"]\n".to_string();
        cargo_toml += "[dependencies]\n";
        let manifest = self.manifest.lock().unwrap();
        for (k, v) in &manifest.dependencies {
            cargo_toml += &format!("{} = \"{}\"\n", k, v);
        }
        fs::write(temp_path.join("Cargo.toml"), cargo_toml)?;

        fs::create_dir_all(temp_path.join("src"))?;
        fs::write(
            temp_path.join("src/lib.rs"),
            "#![allow(unused)]\npub mod content;",
        )?;

        let sources = self.sources.lock().unwrap();
        for (path, code) in &*sources {
            let file_path = temp_path.join(path);
            if let Some(p) = file_path.parent() {
                fs::create_dir_all(p)?;
            }
            fs::write(file_path, code)?;
        }

        let output = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(temp_path)
            .output()?;
        if !output.status.success() {
            anyhow::bail!("cargo failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        let so_bytes = fs::read(temp_path.join("target/release/liblivebloom_core.so"))?;

        let name = CStr::from_bytes_with_nul(b"livebloom_module\0")?;
        let owned_fd: OwnedFd = memfd_create(name, MemFdCreateFlag::MFD_CLOEXEC)?;
        write(&owned_fd, &so_bytes)?;
        let raw_fd = owned_fd.as_raw_fd();
        mem::forget(owned_fd);

        let lib_path = format!("/proc/self/fd/{}", raw_fd);
        let lib = unsafe { Library::new(&lib_path)? };

        {
            let mut modules = self.loaded_modules.lock().unwrap();
            modules.insert(module_name.to_string(), lib);
        }

        println!(
            "🌸 Module '{}' recompilé + swappé EN MÉMOIRE !",
            module_name
        );
        Ok(())
    }

    pub fn get_html(&self, mode: i32) -> String {
        let modules = self.loaded_modules.lock().unwrap();
        if let Some(lib) = modules.get("core") {
            let mut buffer = [0u8; 1024];
            let mut len = 0usize;
            unsafe {
                if let Ok(func) =
                    lib.get::<unsafe extern "C" fn(i32, *mut u8, *mut usize)>(b"get_html")
                {
                    func(mode, buffer.as_mut_ptr(), &mut len as *mut usize);
                    return String::from_utf8_lossy(&buffer[0..len]).to_string();
                }
            }
        }
        "<h1>Chargement...</h1>".to_string()
    }
}
