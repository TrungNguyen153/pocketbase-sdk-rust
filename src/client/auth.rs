use super::PocketBase;
use crate::{
    error::{Error, Result},
    user::{AdminResponse, User, UserResponse, UserTypes},
};
use log::debug;
use reqwest::Response;
use std::collections::HashMap;

impl PocketBase {
    pub async fn auth_via_email<S: AsRef<str>>(
        &mut self,
        email: S,
        password: S,
        usertype: UserTypes,
    ) -> Result<()> {
        match usertype {
            UserTypes::User => {
                self.authenticate_as_user(email.as_ref(), password.as_ref())
                    .await
            }
            UserTypes::Admin => {
                self.authenticate_as_admin(email.as_ref(), password.as_ref())
                    .await
            }
        }
    }

    async fn authenticate_as_user<S: Into<String>>(&mut self, email: S, password: S) -> Result<()> {
        let mut credentials: HashMap<String, String> = HashMap::new();
        credentials.insert("identity".to_string(), email.into());
        credentials.insert("password".to_string(), password.into());

        let response = self
            .send_post("/api/collections/users/auth-with-password", &credentials)
            .await?;

        self.resolve_authorization_response(response, UserTypes::User)
            .await?;

        Ok(())
    }

    async fn authenticate_as_admin<S: Into<String>>(
        &mut self,
        email: S,
        password: S,
    ) -> Result<()> {
        let mut credentials: HashMap<String, String> = HashMap::new();
        credentials.insert("identity".to_string(), email.into());
        credentials.insert("password".to_string(), password.into());

        let response = self
            .send_post("/api/admin/auth-with-password", &credentials)
            .await?;

        self.resolve_authorization_response(response, UserTypes::Admin)
            .await?;

        Ok(())
    }

    async fn resolve_authorization_response(
        &mut self,
        response: Response,
        usertype: UserTypes,
    ) -> Result<()> {
        // let body = response.text().await.unwrap();
        // debug!("result={body}");

        match usertype {
            UserTypes::User => match response.json::<UserResponse>().await {
                Ok(user) => {
                    self.user = Some(User::new_user(user.token, usertype, user.record));
                }
                Err(e) => return Err(Error::AuthenticationError(Box::new(e))),
            },
            UserTypes::Admin => match response.json::<AdminResponse>().await {
                Ok(admin) => {
                    self.user = Some(User::new_admin(admin.token, usertype, admin.admin));
                }
                Err(e) => return Err(Error::AuthenticationError(Box::new(e))),
            },
        };

        debug!("Authentication success with user: {:#?}", self.user);

        Ok(())
    }

    pub async fn refresh_token(&mut self) -> Result<()> {
        if self.user.is_none() {
            return Err(Error::NotAuthenticated);
        }
        let response = self
            .send_post(
                "/api/collections/users/auth-refresh",
                &HashMap::<String, String>::default(),
            )
            .await?;
        self.resolve_authorization_response(response, self.user.to_owned().unwrap().usertype)
            .await?;

        Ok(())
    }

    pub fn is_auth_store_valid(&self) -> bool {
        if let Some(user) = &self.user {
            return user.is_valid();
        }
        false
    }
}
