use std::error::Error;

pub mod proto;

pub fn to_internal(e: impl Error) -> tonic::Status {
    tonic::Status::internal(e.to_string())
}
