use std::ffi::OsString;

pub trait Operation {
    //    fn new(req_rcv: Receiver<OsString>, resp_snd: Sender<OsString>);
    fn handle(&mut self, req: OsString);
}

pub struct FileListing {
    //    resp_send: Sender<OsString>,
//    req_rcv: Receiver<OsString>,
}

impl Operation for FileListing {
    fn handle(&mut self, req: OsString) {
        println!("{:?}", req);
    }
}