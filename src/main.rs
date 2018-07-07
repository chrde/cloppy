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
use dispatcher::GuiDispatcher;
use dispatcher::UiAsyncMessage;
use errors::failure_to_string;
use gui::Wnd;
use plugin::State;
use plugin_handler::PluginHandler;
use std::io;
use std::sync::Arc;
use std::thread;

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
mod plugin_handler;

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
    let files = Arc::new(file_listing::FileListing::create(arena, req_snd.clone()));
    let dispatcher_ui = Box::new(GuiDispatcher::new(files.clone(), Box::new(State::default()), req_snd));
    thread::spawn(move || {
        gui::init_wingui(dispatcher_ui).unwrap();
    });
    let wnd = wait_for_wnd(req_rcv.clone()).expect("Didnt receive START msg with main_wnd");
    let mut handler = PluginHandler::new(wnd, files);
    handler.run_forever(req_rcv);
    Ok(0)
}

fn wait_for_wnd(receiver: channel::Receiver<UiAsyncMessage>) -> Option<Wnd> {
    loop {
        let msg = match receiver.recv() {
            Some(e) => e,
            None => {
                println!("Channel closed. Probably UI thread exit.");
                return None;
            }
        };
        if let UiAsyncMessage::Start(wnd) = msg {
            println!("Got wnd");
            return Some(wnd);
        }
    }
}

