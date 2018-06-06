use std::collections::HashMap;
use std::path::Path;
use std::ffi::OsString;
use std::mem;
use winapi::shared::ntdef::LPCWSTR;
use std::cell::RefCell;
use gui::get_string;
use winapi::um::shellapi::SHGetFileInfoW;
use winapi::um::winnt::FILE_ATTRIBUTE_NORMAL;
use winapi::um::shellapi::SHGFI_SYSICONINDEX;
use winapi::um::shellapi::SHGFI_SMALLICON;
use winapi::um::shellapi::SHGFI_USEFILEATTRIBUTES;
use winapi::um::shellapi::SHFILEINFOW;
use winapi::um::winnt::FILE_ATTRIBUTE_DIRECTORY;
use winapi::um::shellapi::SHGFI_ICON;
use gui::list_view::Item;

pub struct IconRetriever {
    cache: RefCell<HashMap<OsString, i32>>,
    image_list: usize,
    default_index: i32,
    directory_index: i32,
}

pub struct Icon {
    pub width: i32,
    pub image_list: usize,
    pub index: i32,
}

impl IconRetriever {
    pub fn create() -> IconRetriever {
        let (image_list, default_index) = image_list();
        let directory_index = directory_index();
        IconRetriever {
            cache: RefCell::new(HashMap::new()),
            image_list,
            default_index,
            directory_index,
        }
    }

    pub fn get(&self, item: &Item) -> Icon {
        let index = if item.is_directory() {
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
        };
        Icon { width: 16, image_list: self.image_list, index }
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

fn image_list() -> (usize, i32) {
    unsafe {
        let mut info = mem::zeroed::<SHFILEINFOW>();
        let image_list = SHGetFileInfoW(
            get_string("file"),
            FILE_ATTRIBUTE_NORMAL,
            &mut info as *mut _,
            mem::size_of::<SHFILEINFOW> as u32,
            SHGFI_SYSICONINDEX | SHGFI_SMALLICON | SHGFI_USEFILEATTRIBUTES);
        (image_list, info.iIcon)
    }
}

