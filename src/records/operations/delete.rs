use crate::{
    client::PocketBase,
    error::{Error, PocketBaseErrorResponse, Result},
};

impl PocketBase {
    pub async fn delete<S: AsRef<str>>(&self, collection: S, id: S) -> Result<()> {
        let response = self
            .send_delete(format!(
                "/api/collections/{}/records/{}",
                collection.as_ref(),
                id.as_ref()
            ))
            .await?;

        let body = response
            .text()
            .await
            .map_err(|e| Error::RequestFailed(Box::new(e)))?;
        if body == "null" {
            return Ok(());
        }

        match serde_json::from_str::<PocketBaseErrorResponse>(&body) {
            Ok(res) => Err(Error::PocketBaseErrorResponse(res)),
            Err(e) => Err(Error::RequestFailed(Box::new(e))),
        }
    }
}
