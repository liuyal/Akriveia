use actix_web::{ error, error::PayloadError, HttpResponse, };
use serde_derive::{ Deserialize, Serialize, };
use std::fmt;
use common::*;
use tokio_postgres::error::Error as TError;

// MUST MATCH AkError
#[derive(Clone, Serialize, Deserialize)]
pub struct AkError {
    pub reason: String,
    pub t: AkErrorType,
}

impl AkError {
    pub fn internal() -> AkError {
        AkError {
            reason: "Internal Server Error".to_owned(),
            t: AkErrorType::Internal,
        }
    }

    pub fn bad_request(reason: &str) -> AkError {
        AkError {
            reason: reason.to_string(),
            t: AkErrorType::BadRequest,
        }
    }

    pub fn not_found() -> AkError {
        AkError {
            reason: "Not Found".to_owned(),
            t: AkErrorType::NotFound,
        }
    }

    pub fn unauthorized() -> AkError {
        AkError {
            reason: "Unauthorized".to_owned(),
            t: AkErrorType::Unauthorized,
        }
    }

    pub fn validation(reason: &str) -> AkError {
        AkError {
            reason: reason.to_string(),
            t: AkErrorType::Validation,
        }
    }
}

impl fmt::Display for AkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&Err::<(), _>(self)).unwrap())
    }
}

impl fmt::Debug for AkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl error::ResponseError for AkError {
    fn error_response(&self) -> HttpResponse {
        match self.t {
            AkErrorType::BadRequest => HttpResponse::BadRequest().finish(),
            AkErrorType::FileUpload => HttpResponse::BadRequest().finish(),
            AkErrorType::Internal => HttpResponse::InternalServerError().finish(),
            AkErrorType::NotFound => HttpResponse::NotFound().finish(),
            AkErrorType::Unauthorized => HttpResponse::Unauthorized().finish(),
            AkErrorType::Validation => HttpResponse::BadRequest().finish(),
            AkErrorType::ConnectionError => HttpResponse::InternalServerError().finish(),
        }
    }
}

impl From<TError> for AkError {
    fn from(other: TError) -> Self {
        match other.into_source() {
            Some(err) => {
                AkError {
                    t: AkErrorType::Validation,
                    reason: err.to_string(),
                }
            },
            None => {
                AkError {
                    t: AkErrorType::Internal,
                    reason: "Unknown database error.".to_owned(),
                }
            }
        }
    }
}

impl From<PayloadError> for AkError {
    fn from(other: PayloadError) -> Self {
        match other {
            _ => {
                AkError {
                    t: AkErrorType::FileUpload,
                    reason: "File upload failure".to_owned(),
                }
            },
        }
    }
}
