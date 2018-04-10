use gui::context_stash::apply_on_window;
use gui::FILE_LIST_ID;
use gui::INPUT_SEARCH_ID;
use gui::STATUS_BAR_ID;
use gui::wnd_proc::Event;
use std::mem;
use std::ptr;
use winapi::shared::minwindef::HIWORD;
use winapi::shared::minwindef::LOWORD;
use winapi::shared::windef::RECT;
use winapi::um::commctrl::GetEffectiveClientRect;
use winapi::um::winuser::SendMessageW;
use winapi::um::winuser::SetWindowPos;
use winapi::um::winuser::SWP_NOMOVE;
use winapi::um::winuser::SWP_NOSIZE;
use winapi::um::winuser::WM_SIZE;

const INPUT_MARGIN: i32 = 5;
const INPUT_HEIGHT: i32 = 20;
const FILE_LIST_Y: i32 = 2 * INPUT_MARGIN + INPUT_HEIGHT;

pub fn on_size(event: Event) {
    let new_width = LOWORD(event.l_param as u32) as i32;
    let _new_height = HIWORD(event.l_param as u32) as i32;
    unsafe {
        apply_on_window(INPUT_SEARCH_ID, |ref wnd| {
            let width = new_width - 2 * INPUT_MARGIN;
            SetWindowPos(wnd.hwnd, ptr::null_mut(), 0, 0, width, INPUT_HEIGHT, SWP_NOMOVE);
        });
        apply_on_window(STATUS_BAR_ID, |ref wnd| {
            SendMessageW(wnd.hwnd, WM_SIZE, 0, 0);
        });
        apply_on_window(FILE_LIST_ID, |ref wnd| {
            let mut rect = mem::zeroed::<RECT>();
            let mut info = [1, 1, 1, 0, 1, STATUS_BAR_ID, 0, 0];
            GetEffectiveClientRect(event.wnd, &mut rect, info.as_mut_ptr());
            SetWindowPos(wnd.hwnd, ptr::null_mut(), 0, 0, new_width, rect.bottom - FILE_LIST_Y, SWP_NOMOVE);
        });
    }
}

pub fn initial() {
    unsafe {
        apply_on_window(INPUT_SEARCH_ID, |ref wnd| {
            SetWindowPos(wnd.hwnd, ptr::null_mut(), INPUT_MARGIN, INPUT_MARGIN, 0, 0, SWP_NOSIZE);
        });
        apply_on_window(FILE_LIST_ID, |ref wnd| {
            SetWindowPos(wnd.hwnd, ptr::null_mut(), 0, FILE_LIST_Y, 0, 0, SWP_NOSIZE);
        });
    }
}