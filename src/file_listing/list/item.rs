use file_listing::file_entity::FileEntity;
use twoway;
use windows::utils::ToWide;

#[derive(Debug)]
pub struct DisplayItem {
    pub name: String,
    pub path: Vec<u16>,
    pub size: Vec<u16>,
    pub matches: Vec<Match>,
    pub flags: u16,
}

impl DisplayItem {
    pub fn new(file: &FileEntity, path: String, query: &str) -> DisplayItem {
        let matches = matches(query, &file.name());
        let size = if file.is_directory() {
            "".to_wide_null()
        } else {
            pretty_size(file.size()).to_wide_null()
        };
        DisplayItem {
            name: file.name().to_owned(),
            path: path.to_wide_null(),
            size,
            matches,
            flags: file.flags(),
        }
    }
    pub fn is_directory(&self) -> bool {
        self.flags & 2 != 0
    }
}

fn pretty_size(bytes_size: i64) -> String {
    let kb = if bytes_size % 1024 != 0 {
        (bytes_size / 1024) + 1
    } else {
        bytes_size / 1024
    };
    let mut result = kb.to_string();
    let len = result.len();
    let dots = (len - 1) / 3;
    for x in 0..dots {
        let pos = (x + 1) * 3;
        result.insert(len - pos, '.');
    }

    result.push_str(" KB");
    result
}


#[derive(Debug)]
pub struct Match {
    pub matched: bool,
    pub text: Vec<u16>,
}

impl Match {
    pub fn matched(text: &str) -> Match {
        Match {
            matched: true,
            text: text.encode_utf16().collect(),
        }
    }
    pub fn unmatched(text: &str) -> Match {
        Match {
            matched: false,
            text: text.encode_utf16().collect(),
        }
    }
}

pub fn matches(needle: &str, haystack: &str) -> Vec<Match> {
    let mut result = Vec::new();

    let mut curr_pos = 0;
    if needle.len() > 0 {
        while let Some(mut next_pos) = twoway::find_str(&haystack[curr_pos..], &needle) {
            next_pos += curr_pos;
            if next_pos > curr_pos {
                result.push(Match::unmatched(&haystack[curr_pos..next_pos]));
            }
            curr_pos = next_pos + needle.len();
            result.push(Match::matched(&haystack[next_pos..curr_pos]));
        }
    }
    if curr_pos != haystack.len() {
        result.push(Match::unmatched(&haystack[curr_pos..haystack.len()]));
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
            assert_eq!(&expected[x].encode_utf16().collect::<Vec<_>>(), &m.text);
        }
        assert_eq!(matches.len(), expected.len());
    }

    #[test]
    fn pretty_size_test() {
        assert_eq!(&"1 KB", &pretty_size(1));
        assert_eq!(&"1 KB", &pretty_size(1023));
        assert_eq!(&"1 KB", &pretty_size(1024));
        assert_eq!(&"2 KB", &pretty_size(1025));
        assert_eq!(&"17.458 KB", &pretty_size(17876333));
        assert_eq!(&"123.456 KB", &pretty_size(126418944));
        assert_eq!(&"123.456.789 KB", &pretty_size(126419751936));
    }
}