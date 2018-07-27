use errors::MyErrorKind::*;
use failure::Error;
use failure::ResultExt;
use ini::Ini;
use slog::Logger;
use std::collections::HashMap;
use std::fs::{
    File,
    OpenOptions,
};
use std::path::PathBuf;
use std::str::FromStr;
use strum::AsStaticRef;
use strum::IntoEnumIterator;
use windows;

#[derive(AsStaticStr, EnumString, EnumIter, Display, Eq, Hash, PartialEq)]
pub enum Setting {
    DbFile,
    WindowXPosition,
    WindowYPosition,
    WindowWidth,
    WindowHeight,
}

impl Setting {
    pub fn default_value(&self) -> &'static str {
        match self {
            Setting::DbFile => "cloppy.db",
            Setting::WindowXPosition => "50",
            Setting::WindowYPosition => "50",
            Setting::WindowWidth => "50",
            Setting::WindowHeight => "50",
        }
    }
}

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

    pub fn get(&self, setting: Setting) -> Result<&str, Error> {
        self.settings.general_section()
            .get(setting.as_static())
            .map(String::as_str)
            .ok_or(Err(WindowsError("Failed to locate %APPDATA%"))?)
    }

    pub fn get_settings(&self) -> HashMap<Setting, String> {
        self.settings.general_section().iter()
            .map(|(k, v)| (Setting::from_str(&k).unwrap(), v.to_string()))
            .collect()
    }

    fn default_settings() -> Ini {
        let mut conf = Ini::new();
        for setting in Setting::iter() {
            conf.with_section(None::<String>)
                .set(setting.to_string(), setting.default_value().to_string());
        }
        conf
    }

    pub fn update_settings(&mut self, mut settings: HashMap<Setting, String>) -> Result<HashMap<Setting, String>, Error> {
        let settings_as_string = settings.drain().map(|(k, v)| (k.to_string(), v)).collect::<HashMap<String, String>>();
        self.settings.general_section_mut().extend(settings_as_string);
        match self.settings.write_to_file(UserSettings::location()?) {
            Ok(_) => {
                Ok(self.settings.general_section().iter()
                    .map(|(k, v)| (Setting::from_str(&k).unwrap(), v.to_string()))
                    .collect())
            }
            Err(e) => Err(e).with_context(|e| format!("Failed to update settings: {}", e))?
        }
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