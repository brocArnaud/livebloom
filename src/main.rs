use axum::{extract::Query, response::Html, routing::get, Router};
use livebloom::LiveBloom;
use serde::Deserialize;
use std::time::Duration;
use tokio::time;
use tracing::{error, info};

#[derive(Deserialize)]
struct PathQuery {
    path: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let bloom = match LiveBloom::new("my_dream_project") {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to create LiveBloom: {}", e);
            return;
        }
    };

    if let Err(e) = bloom.rebuild_and_swap("core") {
        error!("Initial build failed: {}", e);
        return;
    }

    let bloom_state = bloom.clone();

    let app = Router::new()
        .route(
            "/",
            get(|| async {
                Html(
                    r#"<html>
<head>
<script src="https://cdnjs.cloudflare.com/ajax/libs/three.js/r134/three.min.js"></script>
<script src="https://unpkg.com/htmx.org@1.9.10/dist/htmx.min.js"></script>
<style>body{font-family:Arial;text-align:center} canvas{border:1px solid #000}</style>
</head>
<body>
<h1>LiveBloom + Three.js</h1>
<canvas id="three" width="400" height="300"></canvas>
<div hx-get="/content" hx-trigger="every 1s" hx-swap="innerHTML"></div>
<a href="/content?path=/hello">/hello</a>
<script>
window._bloomAnimId = null;
let scene = new THREE.Scene();
let camera = new THREE.PerspectiveCamera(75, 400/300, 0.1, 1000);
let renderer = new THREE.WebGLRenderer({canvas: document.getElementById('three')});
camera.position.z = 5;
let geometry = new THREE.BoxGeometry();
let material = new THREE.MeshBasicMaterial({color: 0x00ff00, wireframe: true});
let cube = new THREE.Mesh(geometry, material);
scene.add(cube);
function animate() { window._bloomAnimId = requestAnimationFrame(animate); cube.rotation.x += 0.01; cube.rotation.y += 0.01; renderer.render(scene, camera); }
animate();
</script>
</body></html>"#,
                )
            }),
        )
        .route(
            "/content",
            get(move |Query(q): Query<PathQuery>| async move {
                let path = q.path.unwrap_or_default();
                if let Some(html) = bloom_state.get_route(&path) {
                    Html(html)
                } else {
                    Html(bloom_state.get_html())
                }
            }),
        );

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind to port 3000: {}", e);
            return;
        }
    };
    info!("Server started → http://localhost:3000");
    let server = axum::serve(listener, app);

    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.await {
            error!("Server error: {}", e);
        }
    });

    // === AGENT DEMO SEQUENCE ===
    run_agents(bloom).await;

    info!("Agent sequence complete. Server still running.");
    // Keep the server alive indefinitely
    if let Err(e) = server_handle.await {
        error!("Server task failed: {}", e);
    }
}

async fn run_agents(bloom: LiveBloom) {
    time::sleep(Duration::from_secs(10)).await;
    info!("AGENT 1: Swap to COUCOU");
    bloom.edit_file(
        "src/content.rs",
        r#"
#[export_name = "get_html"]
pub extern "C" fn get_html() -> *mut std::ffi::c_char {
    let html = "<h1>🌸 COUCOU ! Le swap a marché sans coupure !</h1>";
    let c_string = std::ffi::CString::new(html).unwrap_or_default();
    c_string.into_raw()
}

#[export_name = "free_html"]
pub unsafe extern "C" fn free_html(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() { let _ = std::ffi::CString::from_raw(ptr); }
}
"#
        .to_string(),
    );
    if let Err(e) = bloom.rebuild_and_swap_async("core").await {
        error!("Agent 1 rebuild failed: {}", e);
    }

    time::sleep(Duration::from_secs(5)).await;
    info!("AGENT 2: Swap to table");
    bloom.edit_file(
        "src/content.rs",
        r#"
#[export_name = "get_html"]
pub extern "C" fn get_html() -> *mut std::ffi::c_char {
    let rows = (0..5).map(|i| format!("<tr><td>Row {}</td><td>Value {}</td></tr>", i, i*7)).collect::<Vec<_>>().join("");
    let html = format!("<table border='1'><tr><th>Index</th><th>Value</th></tr>{}</table>", rows);
    let c_string = std::ffi::CString::new(html).unwrap_or_default();
    c_string.into_raw()
}

#[export_name = "free_html"]
pub unsafe extern "C" fn free_html(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() { let _ = std::ffi::CString::from_raw(ptr); }
}
"#
        .to_string(),
    );
    if let Err(e) = bloom.rebuild_and_swap_async("core").await {
        error!("Agent 2 rebuild failed: {}", e);
    }

    time::sleep(Duration::from_secs(5)).await;
    info!("AGENT 3: Add rand + random values");
    bloom.add_dependency("rand".to_string(), "0.8".to_string());
    bloom.edit_file(
        "src/content.rs",
        r#"
use rand::Rng;

#[export_name = "get_html"]
pub extern "C" fn get_html() -> *mut std::ffi::c_char {
    let mut rng = rand::thread_rng();
    let rows = (0..5).map(|i| format!("<tr><td>Row {}</td><td>Random {}</td></tr>", i, rng.gen_range(0..100))).collect::<Vec<_>>().join("");
    let html = format!("<table border='1'><tr><th>Index</th><th>Value</th></tr>{}</table>", rows);
    let c_string = std::ffi::CString::new(html).unwrap_or_default();
    c_string.into_raw()
}

#[export_name = "free_html"]
pub unsafe extern "C" fn free_html(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() { let _ = std::ffi::CString::from_raw(ptr); }
}
"#
        .to_string(),
    );
    if let Err(e) = bloom.rebuild_and_swap_async("core").await {
        error!("Agent 3 rebuild failed: {}", e);
    }

    time::sleep(Duration::from_secs(5)).await;
    info!("AGENT 4: Add /hello route dynamically");
    bloom.add_route(
        "/hello".to_string(),
        "<h1>Hello from agent 4 ! (route added live)</h1>".to_string(),
    );

    time::sleep(Duration::from_secs(5)).await;
    info!("AGENT 5: Swap Three.js scene to pink wireframe sphere");
    bloom.edit_file("src/content.rs", r####"
#[export_name = "get_html"]
pub extern "C" fn get_html() -> *mut std::ffi::c_char {
    let html = r#"<script>
if (!window._bloomSphereLoaded) {
    window._bloomSphereLoaded = true;
    setTimeout(() => {
        const canvas = document.getElementById('three');
        if (!canvas) return;
        if (window._bloomAnimId) { cancelAnimationFrame(window._bloomAnimId); window._bloomAnimId = null; }
        let scene = new THREE.Scene();
        let camera = new THREE.PerspectiveCamera(75, 400/300, 0.1, 1000);
        let renderer = new THREE.WebGLRenderer({canvas: canvas});
        camera.position.z = 5;
        let geometry = new THREE.SphereGeometry(1, 32, 32);
        let material = new THREE.MeshBasicMaterial({color: 0xff00ff, wireframe: true});
        let sphere = new THREE.Mesh(geometry, material);
        scene.add(sphere);
        function animate() { window._bloomAnimId = requestAnimationFrame(animate); sphere.rotation.x += 0.01; sphere.rotation.y += 0.01; renderer.render(scene, camera); }
        animate();
        console.log('🌸 Three.js sphere loaded successfully !');
    }, 800);
}
</script>"#;
    let c_string = std::ffi::CString::new(html).unwrap_or_default();
    c_string.into_raw()
}

#[export_name = "free_html"]
pub unsafe extern "C" fn free_html(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() { let _ = std::ffi::CString::from_raw(ptr); }
}
"####.to_string());
    if let Err(e) = bloom.rebuild_and_swap_async("core").await {
        error!("Agent 5 rebuild failed: {}", e);
    }
}
