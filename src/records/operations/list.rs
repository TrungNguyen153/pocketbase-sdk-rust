use std::collections::HashMap;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    client::PocketBase,
    error::{Error, Result},
};

use super::GeneralPocketBaseResponse;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedRecordList<T> {
    pub page: u32,
    pub per_page: u32,
    pub total_items: u32,
    pub total_pages: u32,
    pub items: Vec<T>,
}

impl PocketBase {
    pub async fn get_list<S: AsRef<str>, T: DeserializeOwned>(
        &self,
        collection: S,
        page: Option<u32>,
        per_page: Option<u32>,
        sort: Option<S>,
        filter: Option<S>,
        expand: Option<S>,
    ) -> Result<PaginatedRecordList<T>> {
        let mut params = HashMap::new();
        if let Some(inner) = page {
            params.insert("page", inner.to_string());
        }
        if let Some(inner) = per_page {
            params.insert("perPage", inner.to_string());
        }
        if let Some(inner) = sort {
            params.insert("sort", inner.as_ref().to_string());
        }
        if let Some(inner) = filter {
            params.insert("filter", inner.as_ref().to_string());
        }
        if let Some(inner) = expand {
            params.insert("expand", inner.as_ref().to_string());
        }
        match self
            .send_get(
                format!("/api/collections/{}/records", collection.as_ref()),
                Some(&params),
            )
            .await?
            .json::<GeneralPocketBaseResponse<PaginatedRecordList<T>>>()
            .await
        {
            Ok(GeneralPocketBaseResponse::SuccessResponse(res)) => Ok(res),
            Ok(GeneralPocketBaseResponse::ErrorResponse(res)) => {
                Err(Error::PocketBaseErrorResponse(res))
            }
            Err(e) => Err(Error::RequestFailed(Box::new(e))),
        }
    }
}
