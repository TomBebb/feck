use failure::Error;
use futures::stream::Concat2;
use futures::{Future, Stream};
use reqwest;
use reqwest::unstable::async::{Chunk, Decoder, Response};
use std::mem;
use tokio_core::reactor::Handle;

/// File storage service
pub trait Service {
    type File: FileMeta;
    fn new(handle: &Handle) -> Box<Future<Item = Self, Error = Error>>;
    fn list_folder(
        &self,
        handle: &Handle,
        path: &str,
    ) -> Box<Future<Item = Vec<Self::File>, Error = Error>>;
}

/// A file metadata on the service. Can be a folder or a file
pub trait FileMeta {
    /// The file name
    fn name(&self) -> &str;
    /// Size of the file in bytes
    fn size(&self) -> Option<u64>;
}

pub fn get_body(mut res: Response) -> impl Future<Item = Chunk, Error = Error> {
    let body = mem::replace(res.body_mut(), Decoder::empty());
    body.concat2().from_err::<Error>()
}
