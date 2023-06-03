use actix_web::http::StatusCode;
use actix_web::{error, Error, HttpResponse};
use alcoholic_jwt::ValidationError;
use std::env::VarError;
use std::fmt::Formatter;

use bson::ser::Error as BsonError;
use mongodb::error::Error as MongoError;

use reqwest::Error as ReqwestError;
use tonic::transport::Error as GrpcConnectError;

use serde::{Deserialize, Serialize};
use tonic::{Code, Status as GrpcStatusError, Status};

use async_graphql::Error as GraphQLError;

#[derive(Debug)]
pub enum AppError {
    Unauthorized,
    ReqwestAPIError(ReqwestError),
    JwksError(ValidationError),
    Mongo(MongoError),
    ConfigError(VarError),
    ClientError(HttpError),
    ServerError(HttpError),
    BsonError(BsonError),
    AppError(HttpError),
    StandardError(String),
    GrpcConnectionError(GrpcConnectError),
    GrpcStatusError(GrpcStatusError),
    GraphQLError(GraphQLError),
}

impl AppError {
    fn convert_grpc_error_to_status(&self, err: &GrpcStatusError) -> StatusCode {
        match err.code() {
            Code::Ok => StatusCode::OK,
            Code::Cancelled => StatusCode::FAILED_DEPENDENCY,
            Code::Unknown => StatusCode::UNPROCESSABLE_ENTITY,
            Code::InvalidArgument => StatusCode::BAD_REQUEST,
            Code::DeadlineExceeded => StatusCode::REQUEST_TIMEOUT,
            Code::NotFound => StatusCode::NOT_FOUND,
            Code::AlreadyExists => StatusCode::CONFLICT,
            Code::PermissionDenied => StatusCode::FORBIDDEN,
            Code::ResourceExhausted => StatusCode::EXPECTATION_FAILED,
            Code::FailedPrecondition => StatusCode::EXPECTATION_FAILED,
            Code::Aborted => StatusCode::UNPROCESSABLE_ENTITY,
            Code::OutOfRange => StatusCode::RANGE_NOT_SATISFIABLE,
            Code::Unimplemented => StatusCode::NOT_FOUND,
            Code::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Code::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            Code::DataLoss => StatusCode::INSUFFICIENT_STORAGE,
            Code::Unauthenticated => StatusCode::UNAUTHORIZED,
        }
    }

    pub fn convert_app_error_to_graphql_error(&self) -> GraphQLError {
        GraphQLError {
            message: self.to_string(),
            source: None,
            extensions: None,
        }
    }

    pub fn convert_status_error_to_graphql_error(status: Status) -> GraphQLError {
        GraphQLError {
            message: format!("{}", status),
            source: None,
            extensions: None,
        }
    }

    pub fn convert_app_error_to_grpc_status(&self) -> Status {
        match self {
            AppError::Unauthorized => Status::unauthenticated("not authorized".to_string()),
            AppError::ReqwestAPIError(error) => Status::internal(error.to_string()),
            AppError::JwksError(error) => Status::permission_denied(error.to_string()),
            AppError::ConfigError(error) => Status::internal(error.to_string()),
            AppError::ClientError(error) => Status::internal(error.to_string()),
            AppError::ServerError(error) => Status::internal(error.to_string()),
            AppError::BsonError(error) => Status::failed_precondition(error.to_string()),
            AppError::AppError(error) => Status::failed_precondition(error.to_string()),
            AppError::StandardError(error) => Status::invalid_argument(error.to_string()),
            AppError::GrpcConnectionError(error) => Status::unavailable(error.to_string()),
            AppError::GrpcStatusError(error) => Status::new(error.code(), error.message()),
            AppError::GraphQLError(error) => Status::internal(error.clone().message),
            AppError::Mongo(error) => Status::failed_precondition(error.to_string()),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Unauthorized => write!(f, "unauthorized"),
            AppError::ReqwestAPIError(err) => write!(f, "external api error: {}", err),
            AppError::JwksError(err) => write!(f, "validation error: {}", err),
            AppError::ConfigError(err) => write!(f, "configuration error: {}", err),
            AppError::ClientError(err) => write!(f, "External Client error: {}", err),
            AppError::ServerError(err) => write!(f, "External Server error: {}", err),
            AppError::BsonError(err) => write!(f, "Bson error: {}", err),
            AppError::AppError(err) => write!(f, "Document not found : {}", err),
            AppError::StandardError(msg) => write!(f, "see message: {}", msg),
            AppError::GrpcConnectionError(error) => {
                write!(f, "grpc client connect error: {}", error)
            }
            AppError::GrpcStatusError(error) => {
                write!(f, "grpc client connect error: {}", error)
            }
            AppError::GraphQLError(error) => {
                write!(f, "graphQL error: {}", error.clone().message)
            }

            AppError::Mongo(error) => {
                write!(f, "mongo error: {}", error.clone().to_string())
            }
        }
    }
}

impl error::ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::ReqwestAPIError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::JwksError(_) => StatusCode::UNAUTHORIZED,
            AppError::ConfigError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ClientError(_) => StatusCode::BAD_REQUEST,
            AppError::ServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BsonError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::AppError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::StandardError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::GrpcConnectionError(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::GrpcStatusError(status) => self.convert_grpc_error_to_status(status),
            AppError::GraphQLError(_error) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Mongo(_error) => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(AppErrorResponse {
            message: self.error_response(),
        })
    }
}

impl AppError {
    fn error_response(&self) -> String {
        match self {
            AppError::Unauthorized => "unauthorized".into(),
            AppError::ReqwestAPIError(err) => err.to_string(),
            AppError::JwksError(err) => err.to_string(),
            AppError::ConfigError(err) => err.to_string(),
            AppError::ClientError(err) => err.clone().message,
            AppError::ServerError(err) => err.clone().message,
            AppError::BsonError(err) => err.to_string(),
            AppError::AppError(err) => err.clone().message,
            AppError::StandardError(msg) => msg.into(),
            AppError::GrpcConnectionError(error) => error.to_string(),
            AppError::GrpcStatusError(error) => error.to_string(),
            AppError::GraphQLError(error) => error.clone().message,
            AppError::Mongo(error) => error.to_string()
        }
    }
}

impl From<Error> for AppError {
    fn from(err: Error) -> Self {
        AppError::StandardError(err.to_string())
    }
}

impl From<ValidationError> for AppError {
    fn from(err: ValidationError) -> Self {
        AppError::JwksError(err)
    }
}

#[derive(Debug, Serialize)]
pub struct AppErrorResponse {
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpError {
    pub status: u16,
    pub message: String,
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "api error: {}", self.message)
    }
}
