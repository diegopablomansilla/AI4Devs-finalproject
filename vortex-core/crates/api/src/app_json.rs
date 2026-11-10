//! JSON extractor that maps Axum rejections to `ApiError` (`400 invalid_input`).

use axum::{
    Json,
    extract::{FromRequest, Request},
};
use serde::de::DeserializeOwned;

use crate::error::ApiError;

/// Like `Json<T>`, but rejections become `application/problem+json`.
pub struct AppJson<T>(pub T);

impl<T, S> FromRequest<S> for AppJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let uri = req.uri().path().to_string();
        match Json::<T>::from_request(req, state).await {
            Ok(Json(inner)) => Ok(AppJson(inner)),
            Err(rejection) => Err(ApiError::from_json_rejection(rejection, Some(uri))),
        }
    }
}
