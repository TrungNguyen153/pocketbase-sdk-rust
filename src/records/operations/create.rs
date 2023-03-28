use serde::{de::DeserializeOwned, Serialize};

use crate::{
    client::PocketBase,
    error::{Error, Result},
};

use super::GeneralPocketBaseResponse;

impl PocketBase {
    pub async fn create<
        S: AsRef<str>,
        T: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned,
    >(
        &self,
        collection: S,
        model: &T,
    ) -> Result<R> {
        match self
            .send_post(
                format!("/api/collections/{}/records", collection.as_ref()),
                model,
            )
            .await?
            .json::<GeneralPocketBaseResponse<R>>()
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
