use failure::Error;
use gui::event::Event;
use gui::Gui;
use settings::Setting;
use std::collections::HashMap;

pub fn restore_windows_position(_event: Event, gui: &Gui) -> Result<(), Error> {
    let wnd = gui.wnd();
    let settings = gui.settings();
    wnd.set_position(
        setting_to_int(Setting::WindowXPosition, settings),
        setting_to_int(Setting::WindowYPosition, settings),
        setting_to_int(Setting::WindowWidth, settings),
        setting_to_int(Setting::WindowHeight, settings),
        0,
    )
}

fn setting_to_int(setting: Setting, settings: &HashMap<Setting, String>) -> i32 {
    settings.get(&setting).map(|s| s.parse().expect("Setting is not an int")).expect("Setting not found")
}