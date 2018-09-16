use failure::Error;
use futures::{Future, Stream};
use reqwest::unstable::async::{Chunk, Decoder, Response};
use std::path::Path;
use std::{fs, mem};
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
    fn download(&self, handle: &Handle, path: &str) -> Box<Future<Item = Vec<u8>, Error = Error>>;
    fn download_to<'a>(
        &self,
        handle: &Handle,
        path: &str,
        local_path: &'a Path,
    ) -> Box<Future<Item = (), Error = Error>> {
        if local_path.exists() {}
        let local_path = local_path.to_owned();
        Box::new(
            self.download(handle, path)
                .and_then(|bytes| fs::write(local_path, &bytes).map_err(Error::from)),
        )
    }
}

/// A file metadata on the service. Can be a folder or a file
pub trait FileMeta {
    /// The file name
    fn name(&self) -> &str;
    /// Size of the file in bytes
    fn size(&self) -> Option<u64>;
    /// Path
    fn path(&self) -> &str;
}

pub fn get_body(mut res: Response) -> impl Future<Item = Chunk, Error = Error> {
    let body = mem::replace(res.body_mut(), Decoder::empty());
    body.concat2().from_err::<Error>()
}
