use std::{
    fs::{create_dir_all, write},
    path::Path,
    sync::mpsc,
    thread,
    time::{self, SystemTime, UNIX_EPOCH},
};

use screenshots::Screen;

use crate::mouse_hook::{add_middle_click_handler, start_mouse_hook};
const SCREENSHOT_DIR: &str = r"D:\Projects\armadillo-learn-desktop-v2\screens";

pub fn ocr_pipeline() {
    add_middle_click_handler(|x, y| {
        println!("middle click at ({x}, {y})");

        if let Err(e) = create_dir_all(SCREENSHOT_DIR) {
            eprintln!("Не удалось создать папку {SCREENSHOT_DIR}: {e}");
            return;
        }

        let screen = match Screen::from_point(x, y) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Экран для точки ({x}, {y}) не найден: {e}");
                return;
            }
        };

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);

        // capture() возвращает ImageBuffer, у которого есть метод save()
        match screen.capture() {
            Ok(image) => {
                let filename = format!("click_{ts}_display_{}.png", screen.display_info.id);
                let path = Path::new(SCREENSHOT_DIR).join(&filename);

                // Сохраняем изображение напрямую методом .save()
                if let Err(e) = image.save(&path) {
                    // path.display() уже возвращает Display-объект, просто выводим его
                    eprintln!("Не удалось сохранить {}: {e}", path.display());
                } else {
                    println!("Сохранено: {}", path.display());
                }
            }
            Err(e) => eprintln!("Ошибка захвата экрана {}: {e}", screen.display_info.id),
        }
    });

    let (tx, rx) = mpsc::channel::<(i32, i32)>();
    thread::spawn(move || {
        while let Ok((_, _)) = rx.recv() {
            // Тяжёлая работа — уже вне хука.
            thread::sleep(time::Duration::from_millis(50));
        }
    });
    add_middle_click_handler(move |x, y| {
        let _ = tx.send((x, y));
    });

    // Хук блокирует поток — обязательно в отдельном.
    let handle = thread::spawn(start_mouse_hook);
    handle.join().unwrap();
}
