use std::sync::Arc;

use tauri::Manager;

use crate::ocr::OcrState;

mod mouse_hook;
mod ocr;
mod ocr_pipeline;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::thread::spawn(|| {
        ocr_pipeline::ocr_pipeline();
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if let Some(overlay) = app.get_webview_window("overlay") {
                if let Err(e) = overlay.set_ignore_cursor_events(true) {
                    println!("Failed to set ignore_cursor_events: {}", e);
                }
            } else {
                println!("Overlay window not found");
            }
            let state = Arc::new(OcrState::init(app.handle()));
            app.manage(state);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
