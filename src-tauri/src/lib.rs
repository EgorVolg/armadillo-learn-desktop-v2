use std::sync::Arc;

use tauri::Manager;

use crate::ocr::OcrState;

mod mouse_hook;
mod ocr;
mod screenshot;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::thread::spawn(|| {
        mouse_hook::start_mouse_hook();
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let _ = app
                .get_webview_window("overlay")
                .expect("no overlay window found")
                .set_ignore_cursor_events(true)?;
            let _ = Arc::new(OcrState::init(app.handle()));
             
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
