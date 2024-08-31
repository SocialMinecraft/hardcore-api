use std::fmt::Formatter;
use tracing::{event, instrument, Level};
use warp::{Rejection, Reply};
use warp::body::BodyDeserializeError;
use warp::cors::CorsForbidden;
use warp::http::StatusCode;
use warp::reject::Reject;
use crate::errors::Error::DatabaseQueryError;

#[derive(Debug)]
pub enum Error {
    DatabaseQueryError,
}

impl Reject for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &*self {
            Error::DatabaseQueryError => write!(f, "Database error"),
        }
    }
}

#[instrument]
pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(Error::DatabaseQueryError) = r.find() {
        event!(Level::ERROR, "Database query error");
        Ok(warp::reply::with_status(
            DatabaseQueryError.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        event!(Level::ERROR, "cors forbidden error: {}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        event!(Level::ERROR, "cannot deserialize request body: {}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(error) = r.find::<Error>() {
        event!(Level::ERROR, "{}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        event!(Level::WARN, "Requested route was not found");
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}