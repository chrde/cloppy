#[derive(Display, Eq, Hash, PartialEq)]
pub enum Setting {
    DbFile,
    WindowXPosition,
    WindowYPosition,
    WindowWidth,
    WindowHeight,
}

impl Setting {
    pub fn value(&self) -> &'static str {
        match *self {
            Setting::DbFile => "database_location",
            _ => "asd"
        }
    }
}

