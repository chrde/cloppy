use gui::event::Event;
use gui::Gui;
use winapi::shared::minwindef::HIWORD;
use winapi::shared::minwindef::LOWORD;
use winapi::um::winuser::SWP_NOMOVE;
use winapi::um::winuser::SWP_NOSIZE;
use winapi::um::winuser::WM_SIZE;

const INPUT_MARGIN: i32 = 5;
const INPUT_HEIGHT: i32 = 20;
const FILE_LIST_Y: i32 = 2 * INPUT_MARGIN + INPUT_HEIGHT;

pub struct LayoutManager {}

impl LayoutManager {
    pub fn new() -> LayoutManager {
        LayoutManager {}
    }
    pub fn initial(&self, gui: &Gui) {
        gui.input_search().wnd().set_position(INPUT_MARGIN, INPUT_MARGIN, 0, 0, SWP_NOSIZE);
        gui.item_list().wnd().set_position(0, FILE_LIST_Y, 0, 0, SWP_NOSIZE);
    }

    pub fn on_size(&self, gui: &Gui, event: Event) {
        let new_width = LOWORD(event.l_param() as u32) as i32;
        let _new_height = HIWORD(event.l_param() as u32) as i32;
        let width = new_width - 2 * INPUT_MARGIN;
        gui.input_search().wnd().set_position(0, 0, width, INPUT_HEIGHT, SWP_NOMOVE);
        gui.status_bar().wnd().send_message(WM_SIZE, 0, 0);
        gui.item_list().wnd().set_position(0, 0, new_width, gui.client_wnd_height() - FILE_LIST_Y, SWP_NOMOVE);
    }
}

