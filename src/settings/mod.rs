use errors::MyErrorKind::*;
use failure::Error;
use ini::Ini;
use settings::definitions::Settings;
use slog::Logger;
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
    logger: Logger,
}

impl UserSettings {
    pub fn terst() -> Result<File, Error> {
        Ok(OpenOptions::new()
            .read(true)
            .write(true)
            .open("asd")?)
    }

    fn load_or_create(logger: &Logger, location: &PathBuf) -> Result<Ini, Error> {
        let mut file = OpenOptions::new()
            .read(true).write(true).create(true)
            .open(location)?;
        let metadata = file.metadata()?;
        if metadata.len() == 0 {
            let ini = UserSettings::default_settings();
            ini.write_to(&mut file)?;
            info!(logger, "settings not found - using defaults";"file" => location.to_str());
            Ok(ini)
        } else {
            info!(logger, "settings loaded"; "file" => location.to_str());
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

    pub fn load(parent_logger: Logger) -> Result<UserSettings, Error> {
        let logger = parent_logger.new(o!("type" =>"settings"));
        let location = UserSettings::location()?;
        let settings = UserSettings::load_or_create(&logger, &location)?;
        Ok(UserSettings { location, settings, logger })
    }

    fn location() -> Result<PathBuf, Error> {
        let mut user_data = windows::locate_user_data()?;
        user_data.push("cloppy.ini");
        Ok(user_data)
    }
}