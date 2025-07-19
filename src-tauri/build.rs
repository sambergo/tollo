fn main() {
    println!("cargo:rerun-if-changed=data/fin.m3u");
    tauri_build::build()
}