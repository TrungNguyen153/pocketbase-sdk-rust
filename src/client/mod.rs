use std::{collections::HashMap, sync::Arc};

use crate::error::Result;
use crate::{error::Error, user::User};
use eventsource_client::Event;
use futures::future::BoxFuture;
use reqwest::Client;
use tokio::sync::Mutex;
use url::Url;

use self::realtime::PocketBaseRealtime;

mod auth;
mod base;
mod realtime;
pub type HashMapSubscription =
    HashMap<String, Box<dyn Fn(Event) -> BoxFuture<'static, ()> + Send + Sync>>;

pub struct PocketBase {
    pub base_url: Url,
    client: Client,
    pub user: Option<User>,
    subscription: Arc<Mutex<HashMapSubscription>>,
    realtime: Option<PocketBaseRealtime>,
}

impl PocketBase {
    pub fn new<S: AsRef<str>>(raw_url: S) -> Result<PocketBase> {
        match Url::parse(raw_url.as_ref()) {
            Ok(url) => Ok(PocketBase {
                base_url: url,
                user: None,
                client: Client::new(),
                subscription: Default::default(),
                realtime: None,
            }),
            Err(e) => Err(Error::InvalidParameter(Box::new(e))),
        }
    }
}
