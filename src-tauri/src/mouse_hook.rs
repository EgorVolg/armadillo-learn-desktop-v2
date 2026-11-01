use std::os::raw::c_void;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, Mutex};

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::MSLLHOOKSTRUCT;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, HHOOK, MSG, SetWindowsHookExW, TranslateMessage,
    UnhookWindowsHookEx, WH_MOUSE_LL, WM_MBUTTONDOWN, WM_MBUTTONUP,
};

pub type MiddleClickHandler = Arc<dyn Fn(i32, i32) + Send + Sync + 'static>;

static HANDLERS: Mutex<Vec<MiddleClickHandler>> = Mutex::new(Vec::new());
static HOOK: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

/// Регистрирует обработчик клика средней кнопкой мыши.
///
/// Обработчик выполняется в потоке хука — пока он не вернётся,
/// вся мышь в системе ждёт. Тяжёлую работу выносите в канал.
pub fn add_middle_click_handler<F>(f: F)
where
    F: Fn(i32, i32) + Send + Sync + 'static,
{
    let mut guard = HANDLERS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.push(Arc::new(f));
}

/// Запускает `WH_MOUSE_LL`-хук. Блокирует поток до `WM_QUIT`.
pub fn start_mouse_hook() {
    let hook = unsafe {
        SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_callback), None, 0)
            .expect("Не удалось установить хук мыши")
    };
    HOOK.store(hook.0, Ordering::SeqCst);

    let mut msg = MSG::default();
    loop {
        let ok = unsafe { GetMessageW(&mut msg, None, 0, 0) };
        if !ok.as_bool() {
            break;
        }
        let _ = unsafe { TranslateMessage(&msg) };
        let _ = unsafe { DispatchMessageW(&msg) };
    }

    let raw = HOOK.load(Ordering::SeqCst);
    unsafe {
        let _ = UnhookWindowsHookEx(HHOOK(raw));
    }
}

unsafe fn get_click_position(lparam: LPARAM) -> Option<(i32, i32)> {
    if lparam.0 == 0 {
        return None;
    }
    let info = unsafe { *(lparam.0 as *const MSLLHOOKSTRUCT) };
    Some((info.pt.x, info.pt.y))
}

unsafe extern "system" fn mouse_callback(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let hook = HHOOK(HOOK.load(Ordering::SeqCst));

    if code < 0 {
        return unsafe { CallNextHookEx(hook, code, wparam, lparam) };
    }

    let msg_id = wparam.0 as u32;

    if msg_id == WM_MBUTTONDOWN {
        if let Some((x, y)) = unsafe { get_click_position(lparam) } {
            let handlers: Vec<MiddleClickHandler> = HANDLERS
                .lock()
                .map(|guard| guard.clone())
                .unwrap_or_default();
            for handler in handlers {
                handler(x, y);
            }
        }
        return LRESULT(1);
    }

    if msg_id == WM_MBUTTONUP {
        return LRESULT(1);
    }

    unsafe { CallNextHookEx(hook, code, wparam, lparam) }
}
