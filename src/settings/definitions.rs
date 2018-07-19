pub enum Settings {
    DbFile
}

impl Settings {
    pub fn value(&self) -> &'static str {
        match *self {
            Settings::DbFile => "database_location"
        }
    }
}

