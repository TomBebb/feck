use failure::Error;
use futures::{Future, Stream, IntoFuture, stream};
use reqwest::unstable::async::{Chunk, Decoder, Response};
use std::path::{Path, PathBuf};
use std::{fs, mem};
use tokio_core::reactor::Handle;
use std::sync::Arc;

/// File storage service
pub trait Service {
    type File: FileMeta;
    fn new(handle: &Handle) -> Box<Future<Item = Self, Error = Error>>;
    fn list_folder(
        &self,
        handle: &Handle,
        path: &str,
    ) -> Box<Stream<Item = Self::File, Error = Error>>;
    fn download(&self, handle: &Handle, path: &str) -> Box<Future<Item = Vec<u8>, Error = Error>>;
    fn download_all<'a>(&self, handle: &Handle, path: &str, local_path: &'a Path) -> Box<Stream<Item = Self::File, Error = Error>> {
        let path: Arc<str> = path.into();
        let local_path: Arc<Path> = local_path.into();
        if !local_path.is_dir() {
            Box::new(stream::once(Err(format_err!("Path must exist: {:?}", local_path.display()))))
        } else {

            Box::new(self.list_folder(handle, &path).and_then(move |file: Self::File| {
                let name = file.name();
                let new_path = format!("{}/{}", path, name);
                if file.is_dir() {
                    let mut local_path: PathBuf = (&*local_path).to_owned();
                    local_path.push(name);
                    Ok(self.download_all(handle, &new_path, &local_path))
                } else {
                    Ok(Box::new(self.download_to(handle, &new_path, &local_path).map(|_| file).into_stream()) as Box<Stream<Item = Self::File, Error = Error>>)
                }
            }).flatten())
        }
    }
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
    fn is_dir(&self) -> bool {
        self.size() == None
    }
}

pub fn get_body(mut res: Response) -> impl Future<Item = Chunk, Error = Error> {
    let body = mem::replace(res.body_mut(), Decoder::empty());
    body.concat2().from_err::<Error>()
}
