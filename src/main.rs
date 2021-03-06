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
#[macro_use]
extern crate enum_primitive;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate ini;
#[macro_use]
extern crate lazy_static;
extern crate num;
extern crate parking_lot;
extern crate rayon;
extern crate rusqlite;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;
extern crate strum;
#[macro_use]
extern crate strum_macros;
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
use errors::MyErrorKind::UserSettingsError;
use failure::Error;
use failure::ResultExt;
use gui::GuiCreateParams;
use gui::Wnd;
use plugin::Plugin;
use plugin::State;
use plugin_handler::PluginHandler;
use settings::UserSettings;
use std::sync::Arc;
use std::thread;

mod windows;
mod ntfs;
mod actions;
mod plugin;
mod sql;
mod logger;
mod settings;
mod errors;
mod gui;
mod resources;
mod dispatcher;
pub mod file_listing;
mod plugin_handler;

fn main() {
    let logger = logger::setup();
    let result = ntfs::parse_operation::run(logger.clone())
        .and_then(|_| try_main(logger.clone()))
        .map_err(failure_to_string);
    match result {
        Ok(code) => ::std::process::exit(code),
        Err(msg) => error!(logger, "Error: {}", msg),
    }
}

fn try_main(logger: slog::Logger) -> Result<i32, Error> {
    let settings = UserSettings::load(logger.clone()).context(UserSettingsError)?;
    let (req_snd, req_rcv) = channel::unbounded();
    let arena = sql::load_all_arena().unwrap();
    let files = Arc::new(file_listing::FileListing::create(arena, req_snd.clone(), &logger));
    let state = State::new("", 0, files.default_plugin_state());

    let logger_ui = logger.new(o!("thread" => "ui"));
    let dispatcher_ui = GuiDispatcher::new(files.clone(), Box::new(state.clone()), req_snd);
    let settings_ui = settings.get_settings();
    thread::Builder::new().name("producer".to_string()).spawn(move || {
        let gui_params = GuiCreateParams {
            logger: Arc::into_raw(Arc::new(logger_ui)),
            dispatcher: Box::into_raw(Box::new(dispatcher_ui)),
            settings: Box::into_raw(Box::new(settings_ui)),
        };
        gui::init_wingui(gui_params).unwrap()
    }).unwrap();
    let wnd = wait_for_wnd(req_rcv.clone()).expect("Didnt receive START msg with main_wnd");
    let mut handler = PluginHandler::new(wnd, files, state);
    handler.run_forever(req_rcv, settings);
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

