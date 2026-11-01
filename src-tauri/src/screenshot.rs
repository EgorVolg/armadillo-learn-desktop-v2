// Импорты для работы с изображениями и Windows API
use image::{ImageBuffer, Rgba, RgbaImage};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleDC, CreateDIBSection, DIB_USAGE, DeleteDC, DeleteObject,
    GET_DEVICE_CAPS_INDEX, GetDC, GetDeviceCaps, ReleaseDC, SRCCOPY, SelectObject,
};
use windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow;
/// Делает скриншот всего экрана и возвращает его в формате RGBA.
///
/// Эта функция использует Windows GDI API для захвата содержимого экрана.
/// Процесс включает:
/// 1. Получение контекста устройства (DC) рабочего стола
/// 2. Создание совместимого контекста устройства в памяти
/// 3. Создание DIB (Device Independent Bitmap) секции для хранения пикселей
/// 4. Копирование содержимого экрана в DIB с помощью BitBlt
/// 5. Конвертация пикселей из формата BGRA (Windows) в RGBA
/// 6. Очистка ресурсов Windows API
///
/// # Returns
/// * `Ok(RgbaImage)` - скриншот в формате RGBA (32 бита на пиксель)
/// * `Err(String)` - описание ошибки при неудачном захвате экрана
///
/// # Safety
/// Функция использует `unsafe` блок для вызова Windows API функций,
/// которые требуют unsafe для работы с C-совместимыми указателями.
pub fn take_screenshot() -> Result<RgbaImage, String> {
    unsafe {
        // === ШАГ 1: Получение контекста устройства (DC) рабочего стола ===
        // GetDesktopWindow возвращает хэндл окна рабочего стола
        let hwnd = GetDesktopWindow();
        // GetDC получает контекст устройства (Device Context) для указанного окна.
        let hdc = GetDC(hwnd);
        if hdc.is_invalid() {
            return Err("Failed to get window DC".to_string());
        }
        // === ШАГ 2: Получение размеров экрана ===
        // GetDeviceCaps возвращает информацию о возможностях устройства.
        // Константа 8 соответствует HORZRES (горизонтальное разрешение в пикселях)
        let width = GetDeviceCaps(hdc, GET_DEVICE_CAPS_INDEX(8)); // HORZRES
        // Константа 10 соответствует VERTRES (вертикальное разрешение в пикселях)
        let height = GetDeviceCaps(hdc, GET_DEVICE_CAPS_INDEX(10)); // VERTRES
        // === ШАГ 3: Создание совместимого контекста устройства в памяти ===
        // CreateCompatibleDC создает новый контекст устройства, совместимый с hdc.
        // Это DC в памяти, который можно использовать для операций рисования без
        // прямого доступа к экрану. Позволяет создавать и манипулировать изображениями.
        let hdc_mem = CreateCompatibleDC(hdc);
        if hdc_mem.is_invalid() {
            return Err("Failed to create compatible DC".to_string());
        }
        // === ШАГ 4: Настройка структуры BITMAPINFO для создания DIB ===
        // BITMAPINFO описывает формат и размеры DIB (Device Independent Bitmap).
        // DIB - это битмап, который не зависит от конкретного устройства вывода.
        let bmi = windows::Win32::Graphics::Gdi::BITMAPINFO {
            bmiHeader: windows::Win32::Graphics::Gdi::BITMAPINFOHEADER {
                // Размер структуры BITMAPINFOHEADER в байтах
                biSize: std::mem::size_of::<windows::Win32::Graphics::Gdi::BITMAPINFOHEADER>()
                    as u32,
                // Ширина битмапа в пикселях
                biWidth: width,
                // Высота битмапа. Отрицательное значение означает "top-down" DIB,
                // где начало координат в левом верхнем углу (как в большинстве графических систем).
                // Положительное значение означало бы "bottom-up" DIB.
                biHeight: -height,
                // Количество плоскостей (всегда 1 для современных систем)
                biPlanes: 1,
                // Количество битов на пиксель. 32 означает RGBA (8 бит на канал).
                biBitCount: 32,
                // Тип сжатия. 0 = BI_RGB (без сжатия, каждый пиксель занимает 4 байта)
                biCompression: 0,
                // Размер изображения в байтах. 0 означает, что Windows вычислит это автоматически.
                biSizeImage: 0,
                // Горизонтальное разрешение в пикселях на метр (0 = не определено)
                biXPelsPerMeter: 0,
                // Вертикальное разрешение в пикселях на метр (0 = не определено)
                biYPelsPerMeter: 0,
                // Количество используемых цветов в цветовой палитре (0 = использовать все)
                biClrUsed: 0,
                // Количество "важных" цветов (0 = все цвета важны)
                biClrImportant: 0,
            },
            // Цветовая палитра (пустая для 32-битных изображений, т.к. палитра не нужна)
            bmiColors: [Default::default()],
        };
        // === ШАГ 5: Создание DIB секции ===
        // CreateDIBSection создает DIB секцию, которая предоставляет прямой доступ
        // к памяти пикселей. Функция выделяет память и возвращает указатель на нее (bits).
        //
        // Параметры:
        // - hdc: контекст устройства для информации о формате
        // - &bmi: структура BITMAPINFO с описанием формата
        // - 0: использование DIB_RGB_COLORS (стандартный RGB формат)
        // - &mut bits: выходной параметр, получит указатель на память пикселей
        // - None: не используется секция данных (для совместимости с DIB)
        // - 0: смещение (не используется)
        let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
        let hbitmap = CreateDIBSection(hdc, &bmi, DIB_USAGE(0), &mut bits, None, 0)
            .map_err(|e| format!("Failed to create DIB section: {:?}", e))?;
        if hbitmap.is_invalid() {
            // Очистка ресурсов при ошибке
            let _ = DeleteDC(hdc_mem);
            let _ = ReleaseDC(hwnd, hdc);
            return Err("Failed to create DIB section".to_string());
        }
        // === ШАГ 6: Выбор битмапа в контекст устройства памяти ===
        // SelectObject устанавливает наш битмап как текущий объект для hdc_mem.
        // Все операции рисования в hdc_mem теперь будут влиять на наш битмап.
        // Возвращаемое значение - предыдущий битмап (hbitmap_old), который нужно восстановить позже.
        let hbitmap_old = SelectObject(hdc_mem, hbitmap);
        if hbitmap_old.is_invalid() {
            // Очистка ресурсов при ошибке
            let _ = DeleteObject(hbitmap);
            let _ = DeleteDC(hdc_mem);
            let _ = ReleaseDC(hwnd, hdc);
            return Err("Failed to select bitmap".to_string());
        }
        // === ШАГ 7: Копирование содержимого экрана ===
        // BitBlt (Bit Block Transfer) копирует блок пикселей из одного DC в другой.
        // Это основная операция захвата экрана.
        //
        // Параметры:
        // - hdc_mem: целевой DC (наш битмап в памяти)
        // - 0, 0: координаты X, Y в целевом DC (левый верхний угол)
        // - width, height: ширина и высота копируемой области
        // - hdc: исходный DC (экран)
        // - 0, 0: координаты X, Y в исходном DC (откуда копировать)
        // - SRCCOPY: операция копирования (без модификации пикселей)
        BitBlt(hdc_mem, 0, 0, width, height, hdc, 0, 0, SRCCOPY)
            .map_err(|e| format!("BitBlt failed: {:?}", e))?;
        // === ШАГ 8: Чтение пикселей из памяти ===
        // bits указывает на начало памяти пикселей DIB секции.
        // from_raw_parts создает безопасный срез из сырого указателя.
        // Размер среза = ширина * высота * 4 байта (по 4 байта на пиксель RGBA).
        let pixels = std::slice::from_raw_parts(bits as *const u8, (width * height * 4) as usize);
        // === ШАГ 9: Конвертация из BGRA в RGBA ===
        // Windows хранит пиксели в формате BGRA (Blue, Green, Red, Alpha),
        // а библиотека image ожидает RGBA. Создаем ImageBuffer и конвертируем каждый пиксель.
        let mut image: RgbaImage = ImageBuffer::new(width as u32, height as u32);
        for y in 0..height as u32 {
            for x in 0..width as u32 {
                // Вычисляем индекс в массиве пикселей
                // Каждый пиксель занимает 4 байта, поэтому умножаем на 4
                let src_idx = ((y * width as u32 + x) * 4) as usize;
                // Получаем mutable ссылку на пиксель в изображении
                let pixel = image.get_pixel_mut(x, y);
                // Конвертируем BGRA в RGBA, меняя порядок каналов:
                // Windows: [B, G, R, A] -> наша структура: [R, G, B, A]
                *pixel = Rgba([
                    pixels[src_idx + 2], // Red (был третьим в BGRA)
                    pixels[src_idx + 1], // Green (был вторым в BGRA)
                    pixels[src_idx],     // Blue (был первым в BGRA)
                    pixels[src_idx + 3], // Alpha (был четвертым, остается на месте)
                ]);
            }
        }
        // === ШАГ 10: Очистка ресурсов Windows API ===
        // Важно восстановить старый битмап в DC перед удалением нашего битмапа.
        // Это предотвращает утечку ресурсов и dangling references.
        SelectObject(hdc_mem, hbitmap_old);
        // Удаляем созданный битмап
        let _ = DeleteObject(hbitmap);
        // Удаляем контекст устройства памяти
        let _ = DeleteDC(hdc_mem);
        // Освобождаем контекст устройства экрана
        let _ = ReleaseDC(hwnd, hdc);
        // Возвращаем захваченное изображение
        Ok(image)
    }
}
