#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Force software rendering to fix lag/black screens on Nvidia / Wayland
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

    prosounds_lib::run()
}
