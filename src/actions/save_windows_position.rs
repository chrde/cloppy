use gui::Wnd;

pub fn save_windows_position(wnd: &Wnd) {
    let rect = wnd.window_rect();
    println!("{}-{}", rect.top, rect.left);
}