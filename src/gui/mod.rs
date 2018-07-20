use actions::Action;
use actions::shortcuts::on_hotkey_event;
use actions::shortcuts::register_global_files;
use actions::show_files_window;
use dispatcher::GuiDispatcher;
use dispatcher::UiAsyncMessage;
use failure::Error;
use gui::event::Event;
use gui::input_field::InputSearch;
use gui::layout_manager::LayoutManager;
use gui::list_view::ItemList;
use gui::msg::Msg;
use gui::status_bar::StatusBar;
use gui::utils::ToWide;
use gui::wnd_proc::wnd_proc;
use parking_lot::Mutex;
use plugin::State;
pub use self::wnd::Wnd;
use slog::Logger;
use std::collections::HashMap;
use std::ptr;
use std::sync::Arc;
use winapi::shared::minwindef::HINSTANCE;
use winapi::shared::minwindef::LRESULT;
use winapi::shared::minwindef::TRUE;
use winapi::shared::ntdef::LPCWSTR;
use winapi::um::commctrl::*;
use winapi::um::objbase::CoInitialize;
use winapi::um::winuser::*;
use winapi::um::winuser::WM_APP;

mod utils;
mod wnd;
pub mod image_list;
mod wnd_class;
mod msg;
mod tray_icon;
pub mod list_view;
mod input_field;
mod status_bar;
mod wnd_proc;
pub mod default_font;
mod accel_table;
mod layout_manager;
pub mod event;
mod list_header;

type WndId = i32;


const STATUS_BAR_ID: WndId = 1;
const INPUT_SEARCH_ID: WndId = 2;
const FILE_LIST_ID: WndId = 3;
const FILE_LIST_HEADER_ID: WndId = 4;
const MAIN_WND_CLASS: &str = "cloppy";
const MAIN_WND_NAME: &str = "Cloppy main window";
const FILE_LIST_NAME: &str = "File list";
const INPUT_TEXT: &str = "Input text";
const STATUS_BAR: &str = "STATUS_BAR";
const WM_SYSTRAYICON: u32 = WM_APP + 1;
pub const WM_GUI_ACTION: u32 = WM_APP + 2;
pub const STATUS_BAR_CONTENT: &str = "SB_CONTENT";

lazy_static! {
    static ref HASHMAP: Mutex<HashMap<&'static str, Vec<u16>>> = {
    let mut m = HashMap::new();
    m.insert("file_name", "file_name".to_wide_null());
    m.insert("", "".to_wide_null());
    m.insert("file_path", "file_path".to_wide_null());
    m.insert("file_size", "file_size".to_wide_null());
    m.insert("file", "file".to_wide_null());
    m.insert(FILE_LIST_NAME, FILE_LIST_NAME.to_wide_null());
    m.insert(INPUT_TEXT, INPUT_TEXT.to_wide_null());
    m.insert(MAIN_WND_NAME, MAIN_WND_NAME.to_wide_null());
    m.insert(MAIN_WND_CLASS, MAIN_WND_CLASS.to_wide_null());
    m.insert(STATUSCLASSNAME, STATUSCLASSNAME.to_wide_null());
    m.insert(STATUS_BAR, STATUS_BAR.to_wide_null());
    m.insert(WC_EDIT, WC_EDIT.to_wide_null());
    m.insert(WC_LISTVIEW, WC_LISTVIEW.to_wide_null());
        Mutex::new(m)
    };
}

pub fn get_string(str: &str) -> LPCWSTR {
    HASHMAP.lock().get(str)
        .unwrap_or_else(|| panic!("get_string - {} not present", str))
        .as_ptr() as LPCWSTR
}

pub fn set_string(str: &'static str, value: String) {
    HASHMAP.lock().insert(str, value.to_wide_null());
}

pub fn init_wingui(logger: Logger, dispatcher: Box<GuiDispatcher>) -> Result<i32, Error> {
    let res = unsafe { IsGUIThread(TRUE) };
    assert_ne!(res, 0);
    wnd_class::WndClass::init_commctrl()?;
    unsafe { CoInitialize(ptr::null_mut()); }
    let class = wnd_class::WndClass::new(get_string(MAIN_WND_CLASS), wnd_proc)?;
    let accel = accel_table::new()?;
    let mut lp_param = GuiCreateParams { logger: Arc::into_raw(Arc::new(logger)), dispatcher: Box::into_raw(dispatcher) };

    let params = wnd::WndParams::builder()
        .window_name(get_string(MAIN_WND_NAME))
        .class_name(class.0)
        .instance(class.1)
        .style(WS_OVERLAPPEDWINDOW)
        .lp_param(&mut lp_param as *mut _ as *mut _)
        .build();
    let wnd = wnd::Wnd::new(params)?;
    wnd.update()?;
    let mut icon = tray_icon::TrayIcon::new(&wnd);
    icon.set_visible()?;
    loop {
        match MSG::get(None)? {
            MSG { message: WM_QUIT, wParam: code, .. } => {
                return Ok(code as i32);
            }
            mut msg => {
                //TODO drop accel
                if !msg.translate_accel(wnd.hwnd, accel) {
                    msg.translate();
                    msg.dispatch();
                }
            }
        }
    }
}

pub struct GuiCreateParams {
    pub dispatcher: *mut GuiDispatcher,
    pub logger: *const Logger,
}

pub struct Gui {
    logger: Logger,
    wnd: Wnd,
    item_list: ItemList,
    input_search: InputSearch,
    status_bar: StatusBar,
    layout_manager: LayoutManager,
    dispatcher: Box<GuiDispatcher>,
}

impl Gui {
    pub fn create(e: Event, instance: Option<HINSTANCE>, dispatcher: Box<GuiDispatcher>, logger: Logger) -> Result<Gui, Error> {
        let input_search = input_field::new(e.wnd(), instance)?;
        let status_bar = status_bar::new(e.wnd(), instance)?;

        let gui = Gui {
            logger,
            wnd: e.wnd(),
            layout_manager: LayoutManager::new(),
            item_list: list_view::create(e.wnd(), instance),
            input_search: InputSearch::new(input_search),
            status_bar: StatusBar::new(status_bar),
            dispatcher,
        };

        register_global_files(&gui.wnd)?;

        gui.layout_manager.initial(&gui);
        default_font::set_font_on_children(&gui.wnd)?;

        gui.dispatcher.send_async_msg(UiAsyncMessage::Start(gui.wnd.clone()));
        gui.dispatcher.send_async_msg(UiAsyncMessage::Ui("".to_string()));

        Ok(gui)
    }

    pub fn on_get_display_info(&mut self, event: Event) {
        self.item_list.display_item(event, self.dispatcher.as_ref());
    }

    pub fn on_exit_size_move(&mut self, event: Event) {
        println!("new size");
    }

    pub fn on_hotkey(&mut self, event: Event) -> Action {
        on_hotkey_event(&self.logger, event)
    }

    pub fn on_custom_draw(&mut self, event: Event) -> LRESULT {
        self.item_list.custom_draw(event, self.dispatcher.as_mut())
    }

    pub fn on_size(&self, event: Event) {
        self.layout_manager.on_size(self, event);
    }

    pub fn on_custom_action(&mut self, event: Event) {
        let new_state: Box<State> = unsafe { Box::from_raw(event.w_param_mut()) };
        self.status_bar.update(&new_state);
        self.item_list.update(&new_state);
        self.dispatcher.set_state(new_state);
    }

    pub fn input_search(&self) -> &InputSearch {
        &self.input_search
    }

    pub fn item_list(&self) -> &ItemList {
        &self.item_list
    }

    pub fn status_bar(&self) -> &StatusBar {
        &self.status_bar
    }

    pub fn client_wnd_height(&self) -> i32 {
        let info = [1, 1, 1, 0, 1, STATUS_BAR_ID, 0, 0];
        let rect = self.wnd.effective_client_rect(info);
        rect.bottom - rect.top
    }

    pub fn handle_action(&mut self, action: Action, event: Event) {
        match action {
            Action::ShowFilesWindow => show_files_window(event),
            Action::DoNothing => {},
        }
    }
}
