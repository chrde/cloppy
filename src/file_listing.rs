use std::ffi::OsString;
use rusqlite::Connection;
use sql::count_files;
use gui::Wnd;
use gui::WM_GUI_ACTION;
use gui::set_string;
use gui::STATUS_BAR_CONTENT;

pub trait Operation {
    //    fn new(req_rcv: Receiver<OsString>, resp_snd: Sender<OsString>);
    fn handle(&mut self, req: OsString, con: &Connection, wnd: &Wnd);
}

pub struct FileListing {
    //    resp_send: Sender<OsString>,
//    req_rcv: Receiver<OsString>,
}

impl Operation for FileListing {
    fn handle(&mut self, req: OsString, con: &Connection, wnd: &Wnd) {
        let x = req.to_string_lossy().to_string() + "%";
        let file_count = count_files(con, &x);
        let status_bar_message = file_count.to_string() + " objects found";
        set_string(STATUS_BAR_CONTENT, status_bar_message);
        wnd.send_message(WM_GUI_ACTION);
//        update_status_bar(&status_bar_message);
    }
}