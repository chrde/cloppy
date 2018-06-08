use file_listing::files::Files;
use file_listing::State;
use gui::context_stash::CONTEXT_STASH;
use gui::context_stash::ThreadLocalData;
use gui::event::Event;
use gui::input_field::InputSearch;
use gui::layout_manager::LayoutManager;
use gui::layout_manager::Size;
use gui::list_view::ItemList;
use gui::msg::Msg;
use gui::status_bar::StatusBar;
use gui::utils::ToWide;
use gui::wnd_proc::wnd_proc;
use Message;
use parking_lot::Mutex;
pub use self::wnd::Wnd;
use StateChange;
use std::collections::HashMap;
use std::io;
use std::ptr;
use std::sync::Arc;
use std::sync::mpsc;
use winapi::shared::minwindef::HINSTANCE;
use winapi::shared::minwindef::LPVOID;
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
mod context_stash;
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
const MAIN_WND_CLASS: &str = "cloppy_class";
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
    HASHMAP.lock().get(str).unwrap().as_ptr() as LPCWSTR
}

pub fn set_string(str: &'static str, value: String) {
    HASHMAP.lock().insert(str, value.to_wide_null());
}

pub fn init_wingui(sender: mpsc::Sender<Message>, arena: Arc<Files>) -> io::Result<i32> {
    let res = unsafe { IsGUIThread(TRUE) };
    assert_ne!(res, 0);
    CONTEXT_STASH.with(|context_stash| {
        *context_stash.borrow_mut() = Some(ThreadLocalData::new(sender));
    });
    wnd_class::WndClass::init_commctrl()?;
    unsafe { CoInitialize(ptr::null_mut()); }
    let class = wnd_class::WndClass::new(get_string(MAIN_WND_CLASS), wnd_proc)?;
    let accel = accel_table::new()?;

    let params = wnd::WndParams::builder()
        .window_name(get_string(MAIN_WND_NAME))
        .class_name(class.0)
        .instance(class.1)
        .style(WS_OVERLAPPEDWINDOW)
        .lp_param(Arc::into_raw(arena) as LPVOID)
        .build();
    let wnd = wnd::Wnd::new(params)?;
    wnd.show(SW_SHOWDEFAULT);
    wnd.update()?;
    let mut icon = tray_icon::TrayIcon::new(&wnd);
    icon.set_visible()?;
    loop {
        match MSG::get(None).unwrap() {
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

pub struct Gui {
    _wnd: Wnd,
    item_list: ItemList,
    input_search: InputSearch,
    status_bar: StatusBar,
    layout_manager: LayoutManager,
    state: Box<State>,
    arena: Arc<Files>,
}

impl Drop for Gui {
    fn drop(&mut self) {
        unreachable!()
    }
}

impl Gui {
    pub fn create(arena: Arc<Files>, e: Event, instance: Option<HINSTANCE>) -> Gui {
        let input_search = input_field::new(e.wnd(), instance).unwrap();
        let status_bar = status_bar::new(e.wnd(), instance).unwrap();

        let gui = Gui {
            _wnd: Wnd { hwnd: e.wnd() },
            layout_manager: LayoutManager::new(),
            item_list: list_view::create(e.wnd(), instance),
            input_search: InputSearch::new(input_search),
            status_bar: StatusBar::new(status_bar),
            state: Box::new(State::new()),
            arena,
        };
        gui.layout_manager.initial(&gui);
        gui
    }

    pub fn on_get_display_info(&mut self, event: Event) {
        self.item_list.prepare_item(event, &self.arena, &self.state)
    }

    pub fn on_draw_item(&mut self, event: Event) {
        self.item_list.draw_item(event, &self.state)
    }

    pub fn on_size(&self, event: Event) {
        self.layout_manager.on_size(self, event);
    }

    pub fn on_custom_action(&mut self, event: Event) {
        let new_state: Box<State> = unsafe { Box::from_raw(event.w_param_mut()) };
        match *new_state.status() {
            StateChange::NEW => {
                self.state = new_state;
            }
            StateChange::UPDATE => {}
        }
        self.status_bar.update(&self.state);
        self.item_list.update(&self.state);
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

    pub fn client_wnd_size(&self) -> Size {
        let info = [1, 1, 1, 0, 1, STATUS_BAR_ID, 0, 0];
        self._wnd.effective_client_rect(info).into()
    }
}
