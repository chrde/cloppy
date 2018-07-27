use actions::ComposedAction;
use actions::SimpleAction;
use dispatcher::GuiDispatcher;
use errors::failure_to_string;
use gui::accel_table::*;
use gui::event::Event;
use gui::FILE_LIST_ID;
use gui::get_string;
use gui::Gui;
use gui::GuiCreateParams;
use gui::tray_icon;
use gui::utils::FromWide;
use gui::WM_GUI_ACTION;
use gui::WM_SYSTRAYICON;
use settings::Setting;
use std::collections::HashMap;
use std::ffi::OsString;
use std::ptr;
use std::sync::Arc;
use winapi::shared::basetsd::LONG_PTR;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::commctrl::*;
use winapi::um::winuser::*;

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
    let input_text = FindWindowExW(event.wnd().hwnd, ptr::null_mut(), get_string(WC_EDIT), ptr::null_mut());
    SendMessageW(input_text, EM_SETSEL as u32, 0, -1);
}

pub unsafe extern "system" fn wnd_proc(wnd: HWND, message: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let event = Event::new(wnd, l_param, w_param);
    match message {
        WM_CLOSE => {
            let gui = &mut *(GetWindowLongPtrW(wnd, GWLP_USERDATA) as *mut ::gui::Gui);
            gui.handle_action(SimpleAction::MinimizeToTray, event);
            0
        }
        WM_DESTROY => {
            let gui = &mut *(GetWindowLongPtrW(wnd, GWLP_USERDATA) as *mut ::gui::Gui);
            gui.handle_action(SimpleAction::ExitApp, event);
            0
        }
        WM_HOTKEY => {
            let gui = &mut *(GetWindowLongPtrW(wnd, GWLP_USERDATA) as *mut ::gui::Gui);
            let action = gui.on_hotkey(event);
            gui.handle_action(action, event);
            0
        }
        WM_EXITSIZEMOVE => {
            let gui = &mut *(GetWindowLongPtrW(wnd, GWLP_USERDATA) as *mut ::gui::Gui);
            let action = gui.on_exit_size_move(event);
            gui.handle_action(action, event);
            0
        }
        WM_CREATE => {
            let instance = Some((*(l_param as LPCREATESTRUCTW)).hInstance);
            let params = &mut *((*(l_param as LPCREATESTRUCTW)).lpCreateParams as *mut GuiCreateParams);

            let logger = (&*Arc::from_raw(params.logger)).clone();
            let dispatcher: Box<GuiDispatcher> = Box::from_raw(params.dispatcher);
            let settings: Box<HashMap<Setting, String>> = Box::from_raw(params.settings);
            let action = match Gui::create(event, instance, dispatcher, logger, *settings) {
                Err(msg) => panic!(failure_to_string(msg)),
                Ok(mut gui) => {
                    SetWindowLongPtrW(wnd, GWLP_USERDATA, Box::into_raw(Box::new(gui)) as LONG_PTR);
                    ComposedAction::RestoreWindow
                }
            };
            let gui = &mut *(GetWindowLongPtrW(wnd, GWLP_USERDATA) as *mut ::gui::Gui);
            gui.handle_action(action, event);
            0
        }
        WM_NOTIFY => {
            let gui = &mut *(GetWindowLongPtrW(wnd, GWLP_USERDATA) as *mut ::gui::Gui);
            match (*(l_param as LPNMHDR)).code {
                LVN_GETDISPINFOW => {
                    gui.on_get_display_info(event);
                    1
                }

                NM_CUSTOMDRAW => {
                    gui.on_custom_draw(event)
                }
                LVN_COLUMNCLICK => {
                    gui.item_list.on_header_click(event);
                    0
                }
                _ => {
                    DefWindowProcW(wnd, message, w_param, l_param)
                }
            }
        }
        WM_SIZE => {
            let gui = &mut *(GetWindowLongPtrW(wnd, GWLP_USERDATA) as *mut ::gui::Gui);
            gui.on_size(event);
            0
        }
        WM_SYSTRAYICON => {
            let gui = &mut *(GetWindowLongPtrW(wnd, GWLP_USERDATA) as *mut ::gui::Gui);
            let action = tray_icon::on_message(event);
            gui.handle_action(action, event);
            0
        }
//        WM_SYSCOMMAND => {
//            println!("{:?}-{:?}-{:?}", message, w_param & 0xFFF0, l_param);
//            0
//
//        }
        WM_COMMAND => {
            let gui = &mut *(GetWindowLongPtrW(wnd, GWLP_USERDATA) as *mut ::gui::Gui);
            match HIWORD(w_param as u32) as u16 {
                EN_CHANGE => {
                    gui.handle_action(SimpleAction::NewInputQuery, event);
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
                            on_select_all(event);
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
            gui.on_custom_action(event);
            0
        }
        _ => DefWindowProcW(wnd, message, w_param, l_param),
    }
}