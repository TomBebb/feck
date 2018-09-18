#[macro_use]
extern crate failure;
extern crate futures;
extern crate preferences;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate hyper;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio;
extern crate tokio_core;

use common::FileMeta;
use common::Service;
use dropbox::Dropbox;
use futures::Future;
use preferences::AppInfo;
use std::path::Path;
use tokio_core::reactor::Core;
use futures::Stream;

mod common;
mod dropbox;

pub const APP_INFO: AppInfo = AppInfo {
    name: "manysync",
    author: "Tom B",
};

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let work = Dropbox::new(&handle).and_then(|dropbox| {
        dropbox
            .list_folder(&handle, "/")
            .and_then(move |file| dropbox.download_to(&handle, file.path(), Path::new(file.name())))
            .collect()
    });
    println!("{:?}", core.run(work));
}
