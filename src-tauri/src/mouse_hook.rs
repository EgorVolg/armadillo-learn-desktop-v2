use std::os::raw::c_void;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, Ordering};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::MSLLHOOKSTRUCT;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, HHOOK, MSG, SetWindowsHookExW, TranslateMessage,
    UnhookWindowsHookEx, WH_MOUSE_LL, WM_MBUTTONDOWN, WM_MBUTTONUP,
};

unsafe fn get_click_position(lparam: LPARAM) -> Option<(i32, i32)> {
    if lparam.0 == 0 {
        return None;
    }
    let info = unsafe { *(lparam.0 as *const MSLLHOOKSTRUCT) };
    Some((info.pt.x, info.pt.y))
}

// Глобальная атомарная переменная для хранения дескриптора установленного хука мыши.
// Используется AtomicPtr, так как дескриптор может быть доступен из разных потоков
// (основной поток с циклом сообщений и callback-функция хука).
static HOOK: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

/// Запускает низкоуровневый глобальный хук мыши (WH_MOUSE_LL).
/// Устанавливает callback-функцию, запускает цикл сообщений Windows и при завершении снимает хук.
/// Функция блокирует текущий поток до получения сообщения о выходе (WM_QUIT).
pub fn start_mouse_hook() {
    // Устанавливаем хук мыши низкого уровня.
    // WH_MOUSE_LL — перехватывает события мыши перед их доставкой целевому приложению.
    // mouse_callback — указатель на функцию-обработчик событий.
    // None, 0 — хук устанавливается для всех потоков текущего рабочего стола.
    let hook = unsafe {
        SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_callback), None, 0)
            .expect("Не удалось установить хук мыши")
    };
    // Сохраняем дескриптор хука в атомарную переменную для доступа из callback-функции.
    HOOK.store(hook.0, Ordering::SeqCst);

    unsafe {
        let mut msg = MSG::default();
        // Цикл сообщений Windows: получает сообщения из очереди потока.
        // Возвращает false при получении WM_QUIT, после чего цикл завершается.
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg); // Преобразует виртуальные клавиши в символы.
            DispatchMessageW(&msg); // Отправляет сообщение соответствующей оконной процедуре.
        }
    }

    // После выхода из цикла сообщений снимаем установленный хук.
    let raw = HOOK.load(Ordering::SeqCst);
    unsafe {
        let _ = UnhookWindowsHookEx(HHOOK(raw));
    }
}

/// Callback-функция хука мыши, вызываемая Windows при каждом событии мыши.
///
/// # Параметры
/// * `code` — код действия хука. Если < 0, хук должен передать управление следующему.
/// * `wparam` — идентификатор сообщения мыши (WM_MOUSEMOVE, WM_LBUTTONDOWN и т.д.).
/// * `lparam` — указатель на структуру MOUSEHOOKSTRUCT с деталями события.
///
/// # Возвращает
/// LRESULT(1) — блокирует дальнейшую обработку события (событие не достигает приложений).
/// CallNextHookEx(...) — передаёт событие следующему хуку в цепочке.
unsafe extern "system" fn mouse_callback(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // Получаем текущий дескриптор хука из атомарной переменной.
    let hook = HHOOK(HOOK.load(Ordering::SeqCst));

    // Если code < 0, событие нужно просто передать дальше без обработки.
    if code < 0 {
        return unsafe { CallNextHookEx(hook, code, wparam, lparam) };
    }

    // Извлекаем код сообщения мыши из wparam.
    if wparam.0 as u32 == WM_MBUTTONDOWN {
        if let Some((x, y)) = unsafe { get_click_position(lparam) } {
            println!("middle click at ({x}, {y})");
        }
        return LRESULT(1);
    }

    if wparam.0 as u32 == WM_MBUTTONUP {
        return LRESULT(1);
    }
    // Перехватываем нажатие и отпускание средней кнопки мыши (колесо).
    // При обнаружении события выводим сообщение и блокируем его (возвращаем 1),
    // чтобы средняя кнопка не срабатывала в других приложениях.
    if wparam.0 as u32 == WM_MBUTTONDOWN || wparam.0 as u32 == WM_MBUTTONUP {
        println!("Mouse hook: MBUTTONDOWN or MBUTTONUP detected");
        return LRESULT(1);
    }

    // Для всех остальных событий мыши передаём управление следующему хуку в цепочке.
    unsafe { CallNextHookEx(hook, code, wparam, lparam) }
}
