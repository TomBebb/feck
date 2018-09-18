use common::{self, FileMeta, Service};
use failure::Error;
use futures::{Future, Stream, IntoFuture};
use preferences::Preferences;
use reqwest::header::{Authorization, Bearer};
use reqwest::unstable::async::Client;
use serde_json::value;
use serde_json::{self, Value};
use std::collections::HashMap;
use std::{io, str};
use tokio_core::reactor::Handle;

const APP_KEY: &'static str = env!("DROPBOX_APP_KEY");
const APP_SECRET: &'static str = env!("DROPBOX_APP_SECRET");

#[derive(Debug, Clone)]
pub struct Dropbox {
    token: String,
}

header! { (ApiArg, "Dropbox-API-Arg") => [String] }

type Prefs = HashMap<String, String>;

impl Service for Dropbox {
    type File = Metadata;
    fn new(handle: &Handle) -> Box<Future<Item = Dropbox, Error = Error>> {
        if let Some(token) = Prefs::load(&::APP_INFO, "config/dropbox")
            .ok()
            .and_then(|prefs| prefs.get("token").cloned())
        {
            Box::new(Ok(Dropbox { token }).into_future())
        } else {
            println!("Copy & paste code from https://www.dropbox.com/oauth2/authorize?response_type=code&client_id={}", APP_KEY);
            let mut code = String::new();
            io::stdin().read_line(&mut code).unwrap();
            let code = code.trim();
            let mut params = HashMap::new();
            params.insert("code", code);
            params.insert("grant_type", "authorization_code");
            params.insert("client_id", APP_KEY);
            params.insert("client_secret", APP_SECRET);
            Box::new(
                Client::new(handle)
                    .post("https://api.dropboxapi.com/oauth2/token")
                    .form(&params)
                    .send()
                    .from_err::<Error>()
                    .and_then(common::get_body)
                    .and_then(|response| {
                        serde_json::from_slice(response.as_ref()).map_err(|_| {
                            format_err!("{}", unsafe {
                                str::from_utf8_unchecked(response.as_ref())
                            })
                        })
                    }).from_err::<Error>()
                    .and_then(|token: OAuth2Token| {
                        let mut prefs = Prefs::default();
                        prefs.insert("token".into(), token.access_token.clone());
                        prefs
                            .save(&::APP_INFO, "config/dropbox")
                            .map_err(Error::from)
                            .map(|_| token)
                    }).map(|token: OAuth2Token| Dropbox {
                        token: token.access_token,
                    }),
            )
        }
    }
    fn list_folder(
        &self,
        handle: &Handle,
        mut path: &str,
    ) -> Box<Stream<Item = Metadata, Error = Error>> {
        path = path.trim();
        if path == "/" {
            path = "";
        }
        let options = ListFolderArg {
            path,
            ..Default::default()
        };
        Box::new(
            Client::new(handle)
                .post("https://api.dropboxapi.com/2/files/list_folder")
                .header(Authorization(Bearer {
                    token: self.token.clone(),
                })).json(&options)
                .send()
                .from_err::<Error>()
                .and_then(common::get_body)
                .and_then(|response| {
                    serde_json::from_slice(response.as_ref()).map_err(|_| {
                        format_err!("{}", unsafe { str::from_utf8_unchecked(response.as_ref()) })
                    })
                }).from_err::<Error>()
                .and_then(|res: Value| {
                    value::from_value::<Vec<Metadata>>(res.get("entries").unwrap().clone())
                        .map_err(Error::from)
                })
        )
    }
    fn download(&self, handle: &Handle, path: &str) -> Box<Future<Item = Vec<u8>, Error = Error>> {
        let options = DownloadArg { path };
        Box::new(
            Client::new(handle)
                .post("https://content.dropboxapi.com/2/files/download")
                .header(Authorization(Bearer {
                    token: self.token.clone(),
                })).header(ApiArg(serde_json::to_string(&options).unwrap()))
                .send()
                .from_err::<Error>()
                .and_then(common::get_body)
                .map(|response| response.as_ref().to_owned()),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
struct DownloadArg<'a> {
    pub path: &'a str,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
struct ListFolderArg<'a> {
    pub path: &'a str,
    pub recursive: bool,
    pub include_media_info: bool,
    pub include_deleted: bool,
    pub include_has_explicit_shared_members: bool,
    pub include_mounted_folders: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Metadata {
    pub name: String,
    pub id: String,
    /// The size in bytes
    pub size: Option<u64>,
    pub path_lower: String,
}

impl FileMeta for Metadata {
    fn name(&self) -> &str {
        &self.name
    }
    fn size(&self) -> Option<u64> {
        self.size
    }
    fn path(&self) -> &str {
        &self.path_lower
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OAuth2Token {
    pub access_token: String,
    pub token_type: String,
    pub account_id: String,
    pub uid: String,
}
