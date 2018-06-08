use winapi::shared::windef::HDC;
use winapi::shared::windef::RECT;
use winapi::um::commctrl::HIMAGELIST;
use winapi::um::commctrl::ILD_TRANSPARENT;
use winapi::um::commctrl::ImageList_Draw;

pub struct ImageList {
    pub handle: HIMAGELIST,
}

impl ImageList {
    pub fn draw_icon(&self, idx: i32, position: RECT, dest: HDC) {
        unsafe {
            ImageList_Draw(self.handle as HIMAGELIST, idx, dest, position.left, position.top, ILD_TRANSPARENT);
        }
    }
}