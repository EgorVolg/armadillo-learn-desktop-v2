use tauri::Manager;

mod mouse_hook;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::thread::spawn(|| {
        mouse_hook::start_mouse_hook();
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let overlay = app.get_webview_window("overlay").unwrap();
            overlay.set_ignore_cursor_events(true).unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
