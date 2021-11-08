use keri::error::Error as KeriError;
use std::str::Utf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    ParseError(String),
    #[error(transparent)]
    Utf8Error(#[from] Utf8Error),
    #[error(transparent)]
    KerioxError(#[from] KeriError),
}
