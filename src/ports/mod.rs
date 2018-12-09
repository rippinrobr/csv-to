use std::error::Error;

#[derive(Debug)]
pub enum ServiceError {
    Other(Box<Error>),
}

pub type Result<T> = std::result::Result<T, ServiceError>;

pub mod configservice;
pub mod inputservice;
pub mod storageservice;