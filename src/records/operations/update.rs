use serde::{de::DeserializeOwned, Serialize};

use crate::{client::PocketBase, error::{Result, Error}};

use super::GeneralPocketBaseResponse;

impl PocketBase {
    pub async fn update<S: AsRef<str>, T: Serialize + DeserializeOwned>(
        &self,
        collection: S,
        id: S,
        model: &T,
    ) -> Result<T> {
        match self
            .send_patch(
                format!(
                    "/api/collections/{}/records/{}",
                    collection.as_ref(),
                    id.as_ref()
                ),
                model,
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
