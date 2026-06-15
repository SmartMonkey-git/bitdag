use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum BitDagError {
    #[error("Got unknown id {0}")]
    UnknownID(String),
}
