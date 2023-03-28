use serde::Deserialize;
use crate::error::PocketBaseErrorResponse;

pub mod view;
pub mod create;
mod delete;
mod update;
mod list;


#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase", untagged)]
enum GeneralPocketBaseResponse<T> {
    SuccessResponse(T),
    ErrorResponse(PocketBaseErrorResponse),
}
