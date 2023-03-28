use log::debug;
use reqwest::{
    header::{self, AUTHORIZATION},
    Response,
};
use serde::Serialize;

use super::PocketBase;
use crate::{
    error::{Error, Result},
    user::UserTypes,
};

impl PocketBase {
    pub async fn send_get<S: AsRef<str>, T: Serialize + ?Sized>(
        &self,
        path: S,
        query: Option<&T>,
    ) -> Result<Response> {
        match self.base_url.join(path.as_ref()) {
            Ok(endpoint) => {
                let mut req = self.client.get(endpoint);
                req = if let Some(inner_quey) = query {
                    req.query(inner_quey)
                } else {
                    req
                };
                let authed_req = match &self.user {
                    Some(user) => match &user.usertype {
                        UserTypes::User => req.header(AUTHORIZATION, user.token.to_string()),
                        UserTypes::Admin => req.header(AUTHORIZATION, user.token.to_string()),
                    },
                    None => req,
                };
                match authed_req.send().await {
                    Ok(response) => Ok(response),
                    Err(e) => Err(Error::RequestFailed(Box::new(e))),
                }
            }
            Err(e) => Err(Error::RequestFailed(Box::new(e))),
        }
    }

    pub async fn send_post<S: AsRef<str>, T: Serialize + Sized>(
        &self,
        path: S,
        body: &T,
    ) -> Result<Response> {
        match self.base_url.join(path.as_ref()) {
            Ok(endpoint) => {
                let body = serde_json::to_string(body).unwrap_or(String::default());
                debug!("{} - body={body}", endpoint.as_ref());
                let req = self
                    .client
                    .post(endpoint)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(body);
                let authed_req = match &self.user {
                    Some(user) => match &user.usertype {
                        UserTypes::User => req.header(AUTHORIZATION, user.token.to_string()),
                        UserTypes::Admin => req.header(AUTHORIZATION, user.token.to_string()),
                    },
                    None => req,
                };
                match authed_req.send().await {
                    Ok(response) => Ok(response),
                    Err(e) => Err(Error::RequestFailed(Box::new(e))),
                }
            }
            Err(e) => Err(Error::RequestFailed(Box::new(e))),
        }
    }

    pub async fn send_patch<S: AsRef<str>, T: Serialize + Sized>(
        &self,
        path: S,
        body: &T,
    ) -> Result<Response> {
        match self.base_url.join(path.as_ref()) {
            Ok(endpoint) => {
                let req = self
                    .client
                    .patch(endpoint)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(serde_json::to_string(body).unwrap_or(String::default()));
                let authed_req = match &self.user {
                    Some(user) => match &user.usertype {
                        UserTypes::User => req.header(AUTHORIZATION, format!("{}", user.token)),
                        UserTypes::Admin => req.header(AUTHORIZATION, format!("{}", user.token)),
                    },
                    None => req,
                };
                match authed_req.send().await {
                    Ok(response) => Ok(response),
                    Err(e) => Err(Error::RequestFailed(Box::new(e))),
                }
            }
            Err(e) => Err(Error::RequestFailed(Box::new(e))),
        }
    }

    pub async fn send_delete<S: AsRef<str>>(&self, path: S) -> Result<Response> {
        match self.base_url.join(path.as_ref()) {
            Ok(endpoint) => {
                let req = self.client.delete(endpoint);
                let authed_req = match &self.user {
                    Some(user) => match &user.usertype {
                        UserTypes::User => req.header(AUTHORIZATION, format!("{}", user.token)),
                        UserTypes::Admin => req.header(AUTHORIZATION, format!("{}", user.token)),
                    },
                    None => req,
                };
                match authed_req.send().await {
                    Ok(response) => Ok(response),
                    Err(e) => Err(Error::RequestFailed(Box::new(e))),
                }
            }
            Err(e) => Err(Error::RequestFailed(Box::new(e))),
        }
    }
}
