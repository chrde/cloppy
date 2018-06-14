#![feature(plugin, custom_attribute, test)]
#![allow(dead_code)]
#![recursion_limit = "1024"]
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
extern crate regex;
extern crate rusqlite;
extern crate test;
extern crate time;
extern crate twoway;
#[macro_use]
extern crate typed_builder;
extern crate winapi;

use crossbeam_channel as channel;
use file_listing::FileListing;
use file_listing::FilesMsg;
use gui::WM_GUI_ACTION;
use plugin::Plugin;
use std::ffi::OsString;
use std::io;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use winapi::shared::minwindef::WPARAM;

mod windows;
mod ntfs;
mod plugin;
mod sql;
//mod user_settings;
mod errors;
mod gui;
mod resources;
pub mod file_listing;

fn main() {
//    let mut con = sql::main();
//    main1(&mut con);
//    sql::create_indices(&con);
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
    let now = Instant::now();
//    arena.sort_by_name();
    println!("total time {:?}", Instant::now().duration_since(now));
    let plugin = Arc::new(file_listing::FileListing::create(arena, req_snd.clone()));
    let plugin_ui = plugin.clone();
    thread::spawn(move || {
        gui::init_wingui(req_snd, plugin_ui).unwrap();
    });
    run_forever(req_rcv, plugin.clone(), plugin);
    Ok(0)
}

fn run_forever(receiver: channel::Receiver<Message>, plugin: Arc<Plugin>, files: Arc<FileListing>) {
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
                let state = plugin.handle_message(v.to_string_lossy().into_owned());
                wnd.post_message(WM_GUI_ACTION, Box::into_raw(state) as WPARAM);
            }
        }
    }
}

//fn main1(con: &mut Connection) {
//    if let Err(e) = ntfs::start(con) {
//        println!("{}", failure_to_string(e));
//    }
//}

pub enum Message {
    Start(gui::Wnd),
    Ui(OsString),
    Files(FilesMsg),
}


