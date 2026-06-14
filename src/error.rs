use thiserror::Error;

#[derive(Error, Debug)]
pub enum BitDagError {
    #[error("Got unknown id {0}")]
    UnknownID(String),
}
