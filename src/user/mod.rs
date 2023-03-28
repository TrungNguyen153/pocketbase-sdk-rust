use serde::{Deserialize, Serialize};

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
}
