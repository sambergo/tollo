fn main() {
    println!("cargo:rerun-if-changed=data/fin.m3u");
    // generate_context! caches decoded icons in OUT_DIR, so Cargo must track
    // the source icons explicitly to refresh them after icon:select.
    println!("cargo:rerun-if-changed=icons");
    tauri_build::build()
}
