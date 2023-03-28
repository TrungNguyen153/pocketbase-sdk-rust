use std::{collections::HashMap, sync::Arc, time::Duration};

use es::{Client, Event, SSE};
use eventsource_client as es;
use futures::{Future, TryStreamExt};
use log::debug;
use reqwest::{header, Url};
use serde_json::json;
use tokio::{
    sync::{
        mpsc::{self, UnboundedSender},
        Mutex,
    },
    task::JoinHandle,
};

use crate::error::{Error, Result};

use super::{HashMapSubscription, PocketBase};

#[derive(Clone)]
pub struct PocketBaseRealtime {
    id: Arc<Mutex<String>>,
    token: String,
    endpont: Url,
    callbacks: Arc<Mutex<HashMapSubscription>>,
    shutdown_send: Option<UnboundedSender<bool>>,
}

impl PocketBaseRealtime {
    pub fn new<S: AsRef<str>>(
        endpont: Url,
        token: S,
        callbacks: Arc<Mutex<HashMapSubscription>>,
    ) -> Self {
        PocketBaseRealtime {
            id: Default::default(),
            endpont,
            callbacks,
            shutdown_send: None,
            token: token.as_ref().to_string(),
        }
    }

    pub async fn get_conenction_id(&self) -> String {
        self.id.lock().await.to_string()
    }

    fn start_connection(&mut self) -> JoinHandle<std::result::Result<(), es::Error>> {
        let url = self.endpont.to_string();
        let callbacks = self.callbacks.clone();
        let id = self.id.clone();
        let token = self.token.clone();
        let (shutdown_send, mut shutdown_recv) = mpsc::unbounded_channel::<bool>();
        self.shutdown_send = Some(shutdown_send);
        tokio::spawn(async move {
            let client = es::ClientBuilder::for_url(url.as_str())?
                .method("GET".to_string())
                .reconnect(
                    es::ReconnectOptions::reconnect(true)
                        .retry_initial(false)
                        .delay(Duration::from_secs(1))
                        .backoff_factor(2)
                        .delay_max(Duration::from_secs(60))
                        .build(),
                )
                .build_http();

            let mut stream = client.stream();
            loop {
                tokio::select! {
                    next_stream = stream.try_next() => {
                        if let Ok(Some(event)) = next_stream {
                            match event {
                                SSE::Event(real_event) => {
                                    if real_event.event_type == "PB_CONNECT" {
                                        if let Some(event_id) = &real_event.id {
                                            let new_id = {
                                                let mut id_unlocked = id.lock().await;
                                                *id_unlocked = event_id.to_string();
                                                (*id_unlocked).clone()
                                            };
                                            let params = {
                                                let keys = {
                                                    let sub_locked = callbacks.lock().await;
                                                    sub_locked.keys().cloned().collect::<Vec<String>>()
                                                };
                                                let mut params = HashMap::new();
                                                params.insert("clientId".to_string(), json!(new_id.clone()));
                                                params.insert("subscriptions".to_string(), json!(keys));
                                                params
                                            };

                                            let client = reqwest::Client::new();
                                            let req = client.post(url.as_str())
                                                .body(serde_json::to_string(&params).unwrap_or(Default::default()))
                                                .header(header::CONTENT_TYPE, "application/json")
                                                .header(header::AUTHORIZATION, token.clone());


                                            let _ = req.send().await;

                                            continue;
                                        }
                                    }
                                    let callbacks_locked = callbacks.lock().await;
                                    for callback in callbacks_locked.values() {
                                        callback(real_event.clone()).await;
                                    }
                                }
                                SSE::Comment(command) => {
                                    debug!("We got command: {command}");
                                }
                            }
                        }
                    }
                    _ = shutdown_recv.recv() => {
                        debug!("Shutdown");
                        return Ok(())
                    }
                }
            }
        })
    }

    pub async fn ensure_connected(&mut self, timeout: Duration) -> Result<String> {
        self.start_connection();

        let now = std::time::Instant::now();
        while std::time::Instant::now() - now < timeout {
            {
                if let Ok(id_unlocked) = self.id.try_lock() {
                    if !id_unlocked.is_empty() {
                        return Ok(id_unlocked.to_string());
                    }
                }
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        self.shutdown();
        Err(Error::Timeout("Overtime wait SSE Id".to_string()))
    }

    fn shutdown(&self) {
        if let Some(shutdown) = &self.shutdown_send {
            let _ = shutdown.send(true);
        }
    }
}

impl Drop for PocketBaseRealtime {
    fn drop(&mut self) {
        self.shutdown();
    }
}

static API_REALTIME: &str = "/api/realtime";
impl PocketBase {
    pub async fn subscribe<S: AsRef<str>, F, Fut>(
        &mut self,
        collection: S,
        record_id: S,
        callback: F,
    ) -> Result<()>
    where
        F: Fn(Event) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let subscribe_to = Self::resolve_subscribe_to(collection, record_id);
        {
            let mut sub_locked = self.subscription.lock().await;
            sub_locked.remove(&subscribe_to);
            sub_locked.insert(
                subscribe_to.clone(),
                Box::new(move |event| Box::pin(callback(event))),
            );
        }
        if self.realtime.is_none() {
            let endpoint = self.base_url.join(API_REALTIME).expect("WTH");

            let realtime = PocketBaseRealtime::new(
                endpoint,
                self.user.as_ref().unwrap().token.clone(),
                self.subscription.clone(),
            );
            self.realtime = Some(realtime);

            let id = self
                .realtime
                .as_mut()
                .unwrap()
                .ensure_connected(Duration::from_millis(3000))
                .await?;
            debug!("We get id {id}");
        } else {
            // let sse client submit it first time
            self.submit_subscriptions().await?;
        }

        Ok(())
    }

    pub async fn unsubscribe<S: AsRef<str>>(&mut self, collection: S, record_id: S) -> Result<()> {
        let subscribe_to = Self::resolve_subscribe_to(collection, record_id);
        {
            let mut sub_locked = self.subscription.lock().await;
            if subscribe_to.is_empty() {
                sub_locked.clear();
            } else {
                sub_locked.remove(&subscribe_to);
            }
        }
        self.submit_subscriptions().await?;
        Ok(())
    }

    #[inline]
    fn resolve_subscribe_to<S: AsRef<str>>(collection: S, record_id: S) -> String {
        if record_id.as_ref().is_empty() || record_id.as_ref() == "*" {
            collection.as_ref().to_string()
        } else {
            format!("{}/{}", collection.as_ref(), record_id.as_ref())
        }
    }

    #[inline]
    async fn get_sse_id(&self) -> Result<String> {
        if let Some(sse_client) = &self.realtime {
            Ok(sse_client.get_conenction_id().await)
        } else {
            Err(Error::SSEClientNotExist)
        }
    }

    async fn submit_subscriptions(&self) -> Result<()> {
        let id = self.get_sse_id().await?;
        if self.realtime.is_none() {
            return Err(Error::SSEClientNotExist);
        }
        let keys = {
            let sub_locked = self.subscription.lock().await;
            sub_locked.keys().cloned().collect::<Vec<String>>()
        };
        let mut params = HashMap::new();
        params.insert("clientId".to_string(), json!(id));
        params.insert("subscriptions".to_string(), json!(keys));
        let response = self.send_post(API_REALTIME, &params).await?;

        match response.text().await {
            Ok(body) => {
                if body.is_empty() {
                    Ok(())
                } else {
                    Err(Error::PocketBaseErrorResponse(
                        serde_json::from_str(&body).unwrap(),
                    ))
                }
            }
            Err(e) => Err(Error::RequestFailed(Box::new(e))),
        }
    }
}
