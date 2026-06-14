use crate::error::BitDagError;

pub type Result<T> = std::result::Result<T, BitDagError>;

pub mod bitdag;
pub mod dag_edges;
pub mod error;
pub mod traits;
