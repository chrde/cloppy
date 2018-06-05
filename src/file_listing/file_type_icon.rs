use std::collections::HashMap;
use std::path::Path;
use std::ffi::OsString;
use winapi::shared::ntdef::LPCWSTR;
use std::cell::RefCell;

pub struct IconRetriever {
    cache: RefCell<HashMap<OsString, i32>>,
    image_list: usize,
    default_index: i32,
}

impl IconRetriever {
    pub fn new(image_list: usize, default_index: i32) -> IconRetriever {
        IconRetriever {
            cache: RefCell::new(HashMap::new()),
            image_list,
            default_index,
        }
    }

    pub fn get<P: AsRef<Path>>(&self, file: P) -> i32 {
        match file.as_ref().extension() {
            None => self.default_index,
            Some(ext) => {
                let mut cache = self.cache.borrow_mut();
                if let Some(cached) = cache.get(ext).map(|i| *i) {
                    cached
                } else {
                    use gui::list_view::image_index_of;
                    use windows::utils::ToWide;
                    let index = image_index_of(file.as_ref().to_wide_null().as_mut_ptr() as LPCWSTR);
                    cache.insert(ext.to_owned(), index);
                    index
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let icons = IconRetriever::new(12, 51);
        icons.get("asd");
    }
}
