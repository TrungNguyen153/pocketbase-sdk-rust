use crate::error::{Error, Result};
use base64::Engine;
use chrono::{DateTime, NaiveDateTime, Utc};
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::Map;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminResponse {
    pub token: String,
    pub admin: AdminRecord,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminRecord {
    pub id: String,
    pub created: String,
    pub updated: String,
    pub email: String,
    pub avatar: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub token: String,
    pub record: UserRecord,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRecord {
    pub id: String,
    pub collection_id: String,
    pub collection_name: String,
    pub created: String,
    pub updated: String,
    pub username: String,
    pub verified: bool,
    pub email_visibility: bool,
    pub email: String,
    pub name: String,
    pub avatar: String,
}
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum UserTypes {
    #[default]
    User,
    Admin,
}

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub token: String,
    pub usertype: UserTypes,
    pub admin_record: Option<AdminRecord>,
    pub user_record: Option<UserRecord>,
}

impl User {
    pub fn new_admin(token: String, user_type: UserTypes, admin_record: AdminRecord) -> Self {
        User {
            token,
            usertype: user_type,
            admin_record: Some(admin_record),
            user_record: None,
        }
    }

    pub fn new_user(token: String, user_type: UserTypes, user_record: UserRecord) -> Self {
        User {
            token,
            usertype: user_type,
            admin_record: None,
            user_record: Some(user_record),
        }
    }

    pub fn is_valid(&self) -> bool {
        if let Ok(expired_datetime) = self.get_expired_datetime() {
            let now = chrono::offset::Utc::now();
            return expired_datetime > now;
        }
        false
    }

    // logic parse from: https://github.com/iluvadev/PocketBaseClient
    pub fn get_expired_datetime(&self) -> Result<DateTime<Utc>> {
        if self.token.is_empty() {
            return Err(Error::ShouldNot(
                "You was moved Token value. This my bad, im too lazy protect it by get/set"
                    .to_string(),
            ));
        }
        let spliter: Vec<&str> = self.token.as_str().split('.').collect();
        if spliter.len() != 3 {
            debug!("token invalid len part: {}", spliter.len());
            return Err(Error::PocketBaseImplementException(
                "Token no longer have 3 part '.' Seem PocketBase Change logic for Token"
                    .to_string(),
            ));
        }

        let mut payload = spliter[1].to_string();

        if payload.len() % 4 == 2 {
            payload += "==";
        } else if payload.len() % 4 == 3 {
            payload += "=";
        }

        debug!("payload string: {payload}");
        if let Ok(decoded_base64) = base64::engine::general_purpose::STANDARD.decode(payload) {
            if let Ok(payload) = String::from_utf8(decoded_base64) {
                debug!("payload decoded string: {payload}");
                if let Ok(json_value) =
                    serde_json::from_str::<Map<String, serde_json::Value>>(&payload)
                {
                    debug!("success parse payload");
                    if let Some(expired_json) = json_value.get("exp") {
                        debug!("payload had json exp: {expired_json}");
                        if let Some(expired) = expired_json.as_i64() {
                            if let Some(native_datetime) =
                                NaiveDateTime::from_timestamp_millis(expired * 1000)
                            {
                                let expired_datetime =
                                    DateTime::<Utc>::from_utc(native_datetime, Utc);
                                debug!("expired at: {expired_datetime}");
                                return Ok(expired_datetime);
                            }
                        }
                    }
                }
            }
        }

        Err(Error::PocketBaseImplementException(
            "Failed parse".to_string(),
        ))
    }
}
