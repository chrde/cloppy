use gui::utils::FromWide;
use std::ffi::OsString;
use std::ptr;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::commctrl::*;
use winapi::um::winuser::*;
use gui::default_font;
use gui::list_view;
use gui::input_field;
use gui::tray_icon;
use gui::FILE_LIST_ID;
use gui::WM_SYSTRAYICON;
use gui::msg::Msg;
use gui::get_string;
use gui::accel_table::*;
use gui::WM_GUI_ACTION;
use gui::context_stash::send_message;
use Message;
use gui::Wnd;
use winapi::shared::basetsd::LONG_PTR;

pub unsafe fn on_select_all(event: Event) {
    let focused_wnd = GetFocus();
    if !focused_wnd.is_null() {
        let mut buffer = [0u16; 20];
        let bytes_read = GetClassNameW(focused_wnd, buffer.as_mut_ptr(), buffer.len() as i32);
        if bytes_read != 0 {
            let class = OsString::from_wide_null(&buffer);
            match class.to_string_lossy().as_ref() {
                WC_EDIT => {
                    SendMessageW(focused_wnd, EM_SETSEL as u32, 0, -1);
                }
                _ => {
                    println!("todo");
                }
            }
        }
    }
    let input_text = FindWindowExW(event.wnd, ptr::null_mut(), get_string(WC_EDIT), ptr::null_mut());
    SendMessageW(input_text, EM_SETSEL as u32, 0, -1);
}

#[derive(Copy, Clone)]
pub struct Event {
    pub wnd: HWND,
    pub l_param: LPARAM,
    pub w_param: WPARAM,
}

pub unsafe extern "system" fn wnd_proc(wnd: HWND, message: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    match message {
        WM_CLOSE => {
            ShowWindow(wnd, SW_HIDE);
            0
        }
        WM_DESTROY => {
            MSG::post_quit(0);
            0
        }
        WM_CREATE => {
            send_message(Message::START(Wnd { hwnd: wnd }));
            let instance = Some((*(l_param as LPCREATESTRUCTW)).hInstance);

            let gui = Box::new(::gui::Gui::create(Event { wnd, l_param, w_param }, instance));
            default_font::set_font_on_children(Event { wnd, l_param, w_param });

            SetWindowLongPtrW(wnd, GWLP_USERDATA, Box::into_raw(gui) as LONG_PTR);
            0
        }
        WM_NOTIFY => {
            let gui = &*(GetWindowLongPtrW(wnd, GWLP_USERDATA) as *const ::gui::Gui);
            match (*(l_param as LPNMHDR)).code {
                LVN_GETDISPINFOW => {
                    list_view::on_get_display_info(Event { wnd, l_param, w_param });
                    1
                }
                LVN_ODCACHEHINT => {
                    list_view::on_cache_hint(Event { wnd, l_param, w_param });
                    0
                }
                LVN_COLUMNCLICK => {
                    gui.item_list.on_header_click(Event { wnd, l_param, w_param });
//                    list_view::on_header_click(Event { wnd, l_param, w_param });
                    0
                }
                _ => {
                    DefWindowProcW(wnd, message, w_param, l_param)
                }
            }
        }
        WM_SIZE => {
            let gui = &*(GetWindowLongPtrW(wnd, GWLP_USERDATA) as *const ::gui::Gui);
            gui.on_size(Event { wnd, l_param, w_param });
//            layout_manager::on_size(Event { wnd, l_param, w_param });
            0
        }
        WM_SYSTRAYICON => {
            tray_icon::on_message(Event { wnd, l_param, w_param });
            0
        }
//        WM_SYSCOMMAND => {
//            println!("{:?}-{:?}-{:?}", message, w_param & 0xFFF0, l_param);
//            0
//
//        }
        WM_COMMAND => {
            match HIWORD(w_param as u32) as u16 {
                EN_CHANGE => {
                    input_field::on_change(Event { wnd, l_param, w_param });
                    InvalidateRect(wnd, ptr::null_mut(), 0);
                    0
                }
                _ => {
                    match LOWORD(w_param as u32) {
                        ID_FILL_LIST => {
                            let list_view = GetDlgItem(wnd, FILE_LIST_ID);
                            SendMessageW(list_view, LVM_SETITEMCOUNT, 2000000, 0);
                            0
                        }
                        ID_SELECT_ALL => {
                            on_select_all(Event { wnd, l_param, w_param });
                            0
                        }
                        _ => DefWindowProcW(wnd, message, w_param, l_param)
                    }
                }
            }
        }
//        WM_RBUTTONUP => {
//            println!("holaa");
//            0
//        }
        WM_GUI_ACTION => {
            let gui = &mut *(GetWindowLongPtrW(wnd, GWLP_USERDATA) as *mut ::gui::Gui);
            gui.on_custom_action(Event { wnd, l_param, w_param });
//            state_update::handle(Event { wnd, l_param, w_param });
            0
        }
        _ => DefWindowProcW(wnd, message, w_param, l_param),
    }
}