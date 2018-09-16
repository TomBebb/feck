use common::{self, FileMeta, Service};
use failure::Error;
use futures::Future;
use reqwest;
use reqwest::header::{Authorization, Bearer, ContentType};
use reqwest::mime;
use reqwest::unstable::async::{Client, Response};
use serde_json::{self, Value};
use serde_json::value;
use std::{io, str};
use std::collections::HashMap;
use std::str::FromStr;
use tokio_core::reactor::Handle;

const APP_KEY: &'static str = env!("DROPBOX_APP_KEY");
const APP_SECRET: &'static str = env!("DROPBOX_APP_SECRET");


pub struct Dropbox {
    token: String
}

impl Service for Dropbox {
    type File = Metadata;
    fn new(handle: &Handle) -> Box<Future<Item=Dropbox, Error=Error>> {
        println!("Copy & paste code from https://www.dropbox.com/oauth2/authorize?response_type=code&client_id={}", APP_KEY);
        let mut code = String::new();
        io::stdin().read_line(&mut code).unwrap();
        let code = code.trim();
        let mut params = HashMap::new();
        params.insert("code", code);
        params.insert("grant_type", "authorization_code");
        params.insert("client_id", APP_KEY);
        params.insert("client_secret", APP_SECRET);
        Box::new(Client::new(handle)
            .post("https://api.dropboxapi.com/oauth2/token")
            .form(&params)
            .send()
            .from_err::<Error>()
            .and_then(common::get_body)
            .and_then(|response|
                serde_json::from_slice(response.as_ref())
                    .map_err(|_| format_err!("{}", unsafe { str::from_utf8_unchecked(response.as_ref()) })))
            .from_err::<Error>()
            .map(|token: OAuth2Token| Dropbox {
                token: token.access_token
            }))
    }
    fn list_folder(&self, handle: &Handle, mut path: &str) -> Box<Future<Item=Vec<Metadata>, Error=Error>> {
        ;
        path = path.trim();
        if path == "/" {
            path = "";
        }
        let options = ListFolderArg {
            path: path.to_string(),
            ..Default::default()
        };
        let req = Client::new(handle)
            .post("https://api.dropboxapi.com/2/files/list_folder")
            .header(Authorization(Bearer { token: self.token.clone() }))
            //.header(ContentType(mime::APPLICATION_JSON))
            .json(&options)
            .send()
            .from_err::<Error>();
        Box::new(req
            .and_then(common::get_body)
            .and_then(|response|
                serde_json::from_slice(response.as_ref())
                    .map_err(|_| format_err!("{}", unsafe { str::from_utf8_unchecked(response.as_ref()) })))
            .from_err::<Error>()
            .and_then(|res: Value| value::from_value::<Vec<Metadata>>(res.get("entries").unwrap().clone()).map_err(Error::from))
        )
    }
}


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ListFolderArg {
    pub path: String,
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
}

impl FileMeta for Metadata {
    fn name(&self) -> &str {
        &self.name
    }
    fn size(&self) -> Option<u64> {
        self.size
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OAuth2Token {
    pub access_token: String,
    pub token_type: String,
    pub account_id: String,
    pub uid: String,
}