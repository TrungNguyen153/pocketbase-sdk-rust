use std::collections::HashMap;

use serde::de::DeserializeOwned;

use crate::{
    client::PocketBase,
    error::{Error, Result},
};

use super::GeneralPocketBaseResponse;

impl PocketBase {
    pub async fn view<S: AsRef<str>, T: DeserializeOwned>(
        &self,
        collection: S,
        id: S,
        expand: Option<S>,
    ) -> Result<T> {
        let mut params = HashMap::new();
        if let Some(inner_expand) = expand {
            params.insert("expand".to_string(), inner_expand.as_ref().to_string());
        }
        match self
            .send_get(
                format!(
                    "/api/collections/{}/record/{}",
                    collection.as_ref(),
                    id.as_ref()
                ),
                if !params.is_empty() {
                    Some(&params)
                } else {
                    None
                },
            )
            .await?
            .json::<GeneralPocketBaseResponse<T>>()
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
