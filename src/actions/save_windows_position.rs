use dispatcher::GuiDispatcher;
use dispatcher::UiAsyncMessage;
use gui::Wnd;
use settings::Setting;
use std::collections::HashMap;

pub fn save_windows_position(wnd: &Wnd, dispatcher: &GuiDispatcher) {
    let rect = wnd.window_rect();
    let mut properties = HashMap::new();
    properties.insert(Setting::WindowXPosition.to_string(), rect.left.to_string());
    properties.insert(Setting::WindowYPosition.to_string(), rect.top.to_string());
    properties.insert(Setting::WindowHeight.to_string(), (rect.bottom - rect.top).to_string());
    properties.insert(Setting::WindowWidth.to_string(), (rect.right - rect.left).to_string());
    dispatcher.send_async_msg(UiAsyncMessage::UpdateSettings(properties));
}