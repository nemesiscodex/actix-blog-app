use deadpool_postgres::PoolError;
use std::fmt;
use tokio_postgres::error::Error;
use tokio_pg_mapper;
use juniper::{IntoFieldError, FieldError, Value};

#[derive(Debug, Clone)]
pub enum AppErrorType {
    DbError,
    #[allow(dead_code)]
    NotFoundError,
    InvalidField
}

#[derive(Debug, Clone)]
pub struct AppError {
    pub message: Option<String>,
    pub cause: Option<String>,
    pub error_type: AppErrorType,
}

impl AppError {
    pub fn message(&self) -> String {
        match &*self {
            AppError {
                message: Some(message),
                ..
            } => message.clone(),
            AppError {
                error_type: AppErrorType::NotFoundError,
                ..
            } => "The requested item was not found".to_string(),
            AppError {
                error_type: AppErrorType::InvalidField,
                ..
            } => "Invalid field value provided".to_string(),
            _ => "An unexpected error has occurred".to_string(),
        }
    }
}

impl IntoFieldError for AppError {
    fn into_field_error(self) -> FieldError { 
        FieldError::new(self.message(), Value::null())
    }
}

impl From<PoolError> for AppError {
    fn from(error: PoolError) -> AppError {
        AppError {
            message: None,
            cause: Some(error.to_string()),
            error_type: AppErrorType::DbError,
        }
    }
}

impl From<Error> for AppError {
    fn from(error: Error) -> AppError {
        AppError {
            message: None,
            cause: Some(error.to_string()),
            error_type: AppErrorType::DbError,
        }
    }
}

impl From<tokio_pg_mapper::Error> for AppError {
    fn from(error: tokio_pg_mapper::Error) -> AppError {
        AppError {
            message: None,
            cause: Some(error.to_string()),
            error_type: AppErrorType::DbError,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.message())
    }
}

#[cfg(test)]
mod tests {

    use super::{AppError, AppErrorType};

    #[test]
    fn test_default_db_error() {
        let db_error = AppError {
            message: None,
            cause: None,
            error_type: AppErrorType::DbError,
        };

        assert_eq!(
            db_error.message(),
            "An unexpected error has occurred".to_string(),
            "Default message should be shown"
        );
    }

    #[test]
    fn test_default_not_found_error() {
        let db_error = AppError {
            message: None,
            cause: None,
            error_type: AppErrorType::NotFoundError,
        };

        assert_eq!(
            db_error.message(),
            "The requested item was not found".to_string(),
            "Default message should be shown"
        );
    }

    #[test]
    fn test_user_db_error() {
        let user_message = "User-facing message".to_string();

        let db_error = AppError {
            message: Some(user_message.clone()),
            cause: None,
            error_type: AppErrorType::DbError,
        };

        assert_eq!(
            db_error.message(),
            user_message,
            "User-facing message should be shown"
        );
    }

}
