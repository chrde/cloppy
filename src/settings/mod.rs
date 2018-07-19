use errors::MyErrorKind::*;
use failure::Error;
use ini::Ini;
use settings::definitions::Settings;
use std::fs::{
    File,
    OpenOptions,
};
use std::path::PathBuf;
use windows;

mod definitions;

pub struct UserSettings {
    location: PathBuf,
    settings: Ini,
}

impl UserSettings {
    pub fn terst() -> Result<File, Error> {
        Ok(OpenOptions::new()
            .read(true)
            .write(true)
            .open("asd")?)
    }

    fn load_or_create(location: &PathBuf) -> Result<Ini, Error> {
        let mut file = OpenOptions::new()
            .read(true).write(true).create(true)
            .open(location)?;
        let metadata = file.metadata()?;
        if metadata.len() == 0 {
            println!("new file - setting defaults");
            let ini = UserSettings::default_settings();
            ini.write_to(&mut file)?;
            Ok(ini)
        } else {
            Ok(Ini::read_from(&mut file)?)
        }
    }

    pub fn get(&self, setting: Settings) -> Result<&str, Error> {
        self.settings.general_section()
            .get(setting.value())
            .map(String::as_str)
            .ok_or(Err(WindowsError("Failed to locate %APPDATA%"))?)
    }

    fn default_settings() -> Ini {
        let mut conf = Ini::new();
        conf.with_section(None::<String>)
            .set("encoding", "utf-8");
        conf.with_section(Some("User".to_owned()))
            .set("given_name", "Tommy")
            .set("family_name", "Green")
            .set("unicode", "Raspberry树莓");
        conf
    }

    pub fn load() -> Result<UserSettings, Error> {
        let location = UserSettings::location()?;
        let settings = UserSettings::load_or_create(&location)?;
        Ok(UserSettings { location, settings })
    }

    fn location() -> Result<PathBuf, Error> {
        let mut user_data = windows::locate_user_data()?;
        user_data.push("cloppy.ini");
        Ok(user_data)
    }
}