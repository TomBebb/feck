#[macro_use]
extern crate failure;
extern crate futures;
extern crate preferences;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio;
extern crate tokio_core;

use common::Service;
use dropbox::Dropbox;
use futures::Future;
use preferences::{AppInfo, Preferences, PreferencesMap};
use tokio_core::reactor::Core;

mod common;
mod dropbox;


pub const APP_INFO: AppInfo = AppInfo { name: "manysync", author: "Tom B" };

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let work = Dropbox::new(&handle).and_then(|dropbox| dropbox.list_folder(&handle, "/"));
    println!("{:?}", core.run(work));
}
