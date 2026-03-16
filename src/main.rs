use axum::{extract::Query, response::Html, routing::get, Router};
use livebloom::LiveBloom;
use serde::Deserialize;
use std::time::Duration;
use tokio::time;

#[derive(Deserialize)]
struct PathQuery {
    path: Option<String>,
}

#[tokio::main]
async fn main() {
    let bloom = LiveBloom::new("my_dream_project");

    bloom.rebuild_and_swap("core").unwrap();

    let bloom_state = bloom.clone();

    let app = Router::new()
        .route("/", get(|| async { Html(r#"<html><head><script src="https://cdnjs.cloudflare.com/ajax/libs/three.js/r134/three.min.js"></script><style>body{font-family:Arial;text-align:center} canvas{border:1px solid #000}</style></head><body><h1>LiveBloom + Three.js</h1><canvas id="three" width="400" height="300"></canvas><div hx-get="/content" hx-trigger="every 1s" hx-swap="innerHTML"></div><a href="/content?path=/hello">/hello</a></body><script>
let scene = new THREE.Scene();
let camera = new THREE.PerspectiveCamera(75, 400/300, 0.1, 1000);
let renderer = new THREE.WebGLRenderer({canvas: document.getElementById('three')});
camera.position.z = 5;
let geometry = new THREE.BoxGeometry();
let material = new THREE.MeshBasicMaterial({color: 0x00ff00, wireframe: true});
let cube = new THREE.Mesh(geometry, material);
scene.add(cube);
function animate() { requestAnimationFrame(animate); cube.rotation.x += 0.01; cube.rotation.y += 0.01; renderer.render(scene, camera); }
animate();
</script>"#) }))
        .route("/content", get(move |Query(q): Query<PathQuery>| async move {
            let path = q.path.unwrap_or_default();
            if let Some(html) = bloom_state.get_route(&path) {
                Html(html)
            } else {
                Html(bloom_state.get_html(0))
            }
        }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🌸 Serveur lancé → http://localhost:3000");
    let server = axum::serve(listener, app);

    tokio::spawn(async move {
        let _ = server.await;
    });

    // === 4 AGENTS + AGENT 5 (Three.js) ===
    time::sleep(Duration::from_secs(10)).await;
    println!("🌸 AGENT 1 : Swap vers COUCOU");
    bloom.edit_file("src/content.rs", r#"
#[export_name = "get_html"]
pub extern "C" fn get_html(mode: i32, html_out: *mut u8, len: *mut usize) {
    eprintln!("🌸 [IN-MEMORY] get_html called for mode {}", mode);
    let html = "<h1>🌸 COUCOU ! Le swap a marché sans coupure !</h1>";
    let bytes = html.as_bytes();
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), html_out, bytes.len()); *len = bytes.len(); }
}
    "#.to_string());
    bloom.rebuild_and_swap("core").unwrap();

    time::sleep(Duration::from_secs(5)).await;
    println!("🌸 AGENT 2 : Swap vers table random");
    bloom.edit_file("src/content.rs", r#"
#[export_name = "get_html"]
pub extern "C" fn get_html(mode: i32, html_out: *mut u8, len: *mut usize) {
    eprintln!("🌸 [IN-MEMORY] get_html called for mode {}", mode);
    let rows = (0..5).map(|i| format!("<tr><td>Row {}</td><td>Value {}</td></tr>", i, i*7)).collect::<Vec<_>>().join("");
    let html = format!("<table border='1'><tr><th>Index</th><th>Value</th></tr>{}</table>", rows);
    let bytes = html.as_bytes();
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), html_out, bytes.len()); *len = bytes.len(); }
}
    "#.to_string());
    bloom.rebuild_and_swap("core").unwrap();

    time::sleep(Duration::from_secs(5)).await;
    println!("🌸 AGENT 3 : Ajoute rand + random réel");
    bloom.add_dependency("rand".to_string(), "0.8".to_string());
    bloom.edit_file("src/content.rs", r#"
use rand::Rng;
#[export_name = "get_html"]
pub extern "C" fn get_html(mode: i32, html_out: *mut u8, len: *mut usize) {
    eprintln!("🌸 [IN-MEMORY] get_html called for mode {}", mode);
    let mut rng = rand::thread_rng();
    let rows = (0..5).map(|i| format!("<tr><td>Row {}</td><td>Random {}</td></tr>", i, rng.gen_range(0..100))).collect::<Vec<_>>().join("");
    let html = format!("<table border='1'><tr><th>Index</th><th>Value</th></tr>{}</table>", rows);
    let bytes = html.as_bytes();
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), html_out, bytes.len()); *len = bytes.len(); }
}
    "#.to_string());
    bloom.rebuild_and_swap("core").unwrap();

    time::sleep(Duration::from_secs(5)).await;
    println!("🌸 AGENT 4 : Ajoute la route /hello dynamiquement !");
    bloom.add_route(
        "/hello".to_string(),
        "<h1>Hello from agent 4 ! (route ajoutée en live)</h1>".to_string(),
    );
    bloom.rebuild_and_swap("core").unwrap();

    time::sleep(Duration::from_secs(5)).await;
    println!("🌸 AGENT 5 : Change la scène Three.js en sphère filaire !");
    bloom.edit_file("src/content.rs", r####"
#[export_name = "get_html"]
pub extern "C" fn get_html(mode: i32, html_out: *mut u8, len: *mut usize) {
    eprintln!("🌸 [IN-MEMORY] get_html called for mode {}", mode);
    let html = r#"<script>
setTimeout(() => {
    const canvas = document.getElementById('three');
    if (!canvas) return;
    // On change juste le mesh existant (pas le canvas)
    let scene = new THREE.Scene();
    let camera = new THREE.PerspectiveCamera(75, 400/300, 0.1, 1000);
    let renderer = new THREE.WebGLRenderer({canvas: canvas});
    camera.position.z = 5;
    let geometry = new THREE.SphereGeometry(1, 32, 32);
    let material = new THREE.MeshBasicMaterial({color: 0xff00ff, wireframe: true});
    let sphere = new THREE.Mesh(geometry, material);
    scene.add(sphere);
    function animate() { requestAnimationFrame(animate); sphere.rotation.x += 0.01; sphere.rotation.y += 0.01; renderer.render(scene, camera); }
    animate();
    console.log('🌸 Three.js sphere loaded successfully !');
}, 800);
</script>"#;
    let bytes = html.as_bytes();
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), html_out, bytes.len()); *len = bytes.len(); }
}
    "####.to_string());
    bloom.rebuild_and_swap("core").unwrap();
}
