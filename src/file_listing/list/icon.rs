use file_listing::list::item::DisplayItem;
use gui::get_string;
use gui::image_list::ImageList;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsString;
use std::mem;
use std::path::Path;
use winapi::shared::ntdef::LPCWSTR;
use winapi::shared::windef::HDC;
use winapi::shared::windef::RECT;
use winapi::um::commctrl::HIMAGELIST;
use winapi::um::shellapi::SHFILEINFOW;
use winapi::um::shellapi::SHGetFileInfoW;
use winapi::um::shellapi::SHGFI_ICON;
use winapi::um::shellapi::SHGFI_SMALLICON;
use winapi::um::shellapi::SHGFI_SYSICONINDEX;
use winapi::um::shellapi::SHGFI_USEFILEATTRIBUTES;
use winapi::um::winnt::FILE_ATTRIBUTE_DIRECTORY;
use winapi::um::winnt::FILE_ATTRIBUTE_NORMAL;

const ICON_PADDING: i32 = 4;

pub struct Icons {
    cache: RefCell<HashMap<OsString, i32>>,
    image_list: ImageList,
    default_index: i32,
    directory_index: i32,
}

type IconWidth = i32;

impl Icons {
    pub fn create() -> Icons {
        let (image_list, default_index) = image_list();
        let directory_index = directory_index();
        Icons {
            cache: RefCell::new(HashMap::new()),
            image_list,
            default_index,
            directory_index,
        }
    }

    pub fn draw_icon(&self, item: &DisplayItem, mut position: RECT, dest: HDC) -> IconWidth {
        let idx = self.get(item);
        position.left += ICON_PADDING;
        self.image_list.draw_icon(idx, position, dest);
        16 + 2 * ICON_PADDING
    }

    pub fn get(&self, item: &DisplayItem) -> i32 {
        if item.is_directory() {
            self.directory_index
        } else {
            let name: &Path = &item.name.as_ref();
            match name.extension() {
                None => self.default_index,
                Some(ext) => {
                    let mut cache = self.cache.borrow_mut();
                    if let Some(cached) = cache.get(ext).map(|i| *i) {
                        cached
                    } else {
                        use windows::utils::ToWide;
                        let index = image_index_of(name.to_wide_null().as_mut_ptr() as LPCWSTR);
                        cache.insert(ext.to_owned(), index);
                        index
                    }
                }
            }
        }
    }
}

pub fn image_index_of(str: LPCWSTR) -> (i32) {
    unsafe {
        let mut info = mem::zeroed::<SHFILEINFOW>();
        SHGetFileInfoW(
            str,
            FILE_ATTRIBUTE_NORMAL,
            &mut info as *mut _,
            mem::size_of::<SHFILEINFOW> as u32,
            SHGFI_SYSICONINDEX | SHGFI_SMALLICON | SHGFI_USEFILEATTRIBUTES);
        info.iIcon
    }
}

pub fn directory_index() -> (i32) {
    unsafe {
        let mut info = mem::zeroed::<SHFILEINFOW>();
        SHGetFileInfoW(
            get_string("file"),
            FILE_ATTRIBUTE_DIRECTORY,
            &mut info as *mut _,
            mem::size_of::<SHFILEINFOW> as u32,
            SHGFI_ICON | SHGFI_SMALLICON | SHGFI_USEFILEATTRIBUTES);
        info.iIcon
    }
}

fn image_list() -> (ImageList, i32) {
    unsafe {
        let mut info = mem::zeroed::<SHFILEINFOW>();
        let handle = SHGetFileInfoW(
            get_string("file"),
            FILE_ATTRIBUTE_NORMAL,
            &mut info as *mut _,
            mem::size_of::<SHFILEINFOW> as u32,
            SHGFI_SYSICONINDEX | SHGFI_SMALLICON | SHGFI_USEFILEATTRIBUTES);
        (ImageList { handle: handle as HIMAGELIST }, info.iIcon)
    }
}

