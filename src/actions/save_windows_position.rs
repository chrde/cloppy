use dispatcher::GuiDispatcher;
use dispatcher::UiAsyncMessage;
use gui::Wnd;
use settings::Setting;
use std::collections::HashMap;

pub fn save_windows_position(wnd: &Wnd, dispatcher: &GuiDispatcher) {
    let rect = wnd.window_rect();
    let mut properties = HashMap::new();
    properties.insert(Setting::WindowXPosition, rect.left.to_string());
    properties.insert(Setting::WindowYPosition, rect.top.to_string());
    properties.insert(Setting::WindowHeight, (rect.bottom - rect.top).to_string());
    properties.insert(Setting::WindowWidth, (rect.right - rect.left).to_string());
    dispatcher.send_async_msg(UiAsyncMessage::UpdateSettings(properties));
}