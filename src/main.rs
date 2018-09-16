#[macro_use]
extern crate failure;
extern crate futures;
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
use tokio_core::reactor::Core;

mod common;
mod dropbox;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let work = Dropbox::new(&handle).and_then(|dropbox| dropbox.list_folder(&handle, "/"));
    println!("{:?}", core.run(work));
}
