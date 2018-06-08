use file_listing::file_entity::FileEntity;
use twoway;
use windows::utils::ToWide;

#[derive(Debug)]
pub struct DisplayItem {
    pub name: String,
    pub path: Vec<u16>,
    pub size: Vec<u16>,
    pub matches: Vec<Match>,
    pub flags: u8,
}

impl DisplayItem {
    pub fn new(file: &FileEntity, path: String, query: &str) -> DisplayItem {
        let matches = matches(query, &file.name());
        DisplayItem {
            name: file.name().to_owned(),
            path: path.to_wide_null(),
            size: file.size().to_string().to_wide_null(),
            matches,
            flags: file.flags(),
        }
    }
    pub fn is_directory(&self) -> bool {
        self.flags & 2 != 0
    }
}

#[derive(Debug)]
pub struct Match {
    pub matched: bool,
    pub init: usize,
    pub end: usize,
}

pub fn matches(needle: &str, haystack: &str) -> Vec<Match> {
    let mut result = Vec::new();

    let mut curr_pos = 0;
    if needle.len() > 0 {
        while let Some(mut next_pos) = twoway::find_str(&haystack[curr_pos..], &needle) {
            next_pos += curr_pos;
            if next_pos > curr_pos {
                result.push(Match { matched: false, init: curr_pos, end: next_pos });
            }
            curr_pos = next_pos + needle.len();
            result.push(Match { matched: true, init: next_pos, end: curr_pos });
        }
    }
    if curr_pos != haystack.len() {
        result.push(Match { matched: false, init: curr_pos, end: haystack.len() });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let haystack = "Część 1 -Część 2";
        let needle = "Czę";
        let matches = matches(needle, haystack);
        let expected = ["Czę", "ść 1 -", "Czę", "ść 2"];
        for x in 0..matches.len() {
            let m = &matches[x];
            assert_eq!(expected[x], &haystack[m.init..m.end])
        }
        assert_eq!(matches.len(), expected.len());
    }
}

