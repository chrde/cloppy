#![feature(plugin, custom_attribute, test)]
#![allow(dead_code)]
#![recursion_limit = "1024"]
//#![feature(rust_2018_preview)]
//#![warn(rust_2018_idioms)]
#[macro_use]
extern crate bitflags;
extern crate byteorder;
extern crate conv;
extern crate core;
extern crate crossbeam_channel;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate ini;
#[macro_use]
extern crate lazy_static;
extern crate parking_lot;
extern crate rayon;
extern crate rusqlite;
extern crate test;
extern crate time;
extern crate twoway;
#[macro_use]
extern crate typed_builder;
extern crate winapi;

use crossbeam_channel as channel;
use dispatcher::Dispatcher;
use dispatcher::GuiDispatcher;
use dispatcher::UiAsyncMessage;
use errors::failure_to_string;
use file_listing::FileListing;
use file_listing::FilesMsg;
use gui::WM_GUI_ACTION;
use plugin::Dummy;
use plugin::Plugin;
use plugin::State;
use std::ffi::OsString;
use std::io;
use std::sync::Arc;
use std::thread;
use winapi::shared::minwindef::WPARAM;

mod windows;
mod ntfs;
mod plugin;
mod sql;
//mod user_settings;
mod errors;
mod gui;
mod resources;
mod dispatcher;
pub mod file_listing;

fn main() {
    if let Err(e) = ntfs::parse_operation::run() {
        panic!("{}", failure_to_string(e));
    }
    match try_main() {
        Ok(code) => ::std::process::exit(code),
        Err(err) => {
            let msg = format!("Error: {}", err);
            panic!(msg);
        }
    }
}

fn try_main() -> io::Result<i32> {
    let (req_snd, req_rcv) = channel::unbounded();
    let arena = sql::load_all_arena().unwrap();
    let plugin = Arc::new(file_listing::FileListing::create(arena, req_snd.clone()));
    let dispatcher = Arc::new(Dispatcher::new(None, plugin, Arc::new(Dummy), req_snd1));
    let dispatcher_ui = dispatcher.clone();
    thread::spawn(move || {
        gui::init_wingui(req_snd, dispatcher_ui).unwrap();
    });
    run_forever(req_rcv, dispatcher);
    Ok(0)
}

fn run_forever(receiver: channel::Receiver<UiAsyncMessage>, dispatcher: Arc<GuiDispatcher>) {
    let mut wnd = None;
    loop {
        let msg = match receiver.recv() {
            Some(e) => e,
            None => {
                println!("Channel closed. Probably UI thread exit.");
                return;
            }
        };
        match msg {
            Message::Start(main_wnd) => wnd = Some(main_wnd),
            Message::Files(msg) => files.on_message(msg),
            Message::Ui(v) => {
                let wnd = wnd.as_mut().expect("Didnt receive START msg with main_wnd");
                let msg = v.to_str().expect("Invalid UI Message");
                let count = plugin.handle_message(msg);
                let state = Box::new(State::new(msg, count));
                wnd.post_message(WM_GUI_ACTION, Box::into_raw(state) as WPARAM);
            }
        }
    }
}


