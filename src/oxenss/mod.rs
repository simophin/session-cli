pub mod batch;
mod error;
pub mod http;
pub mod message;
pub mod namespace;
pub mod retrieve;
pub mod retrieve_service_node;
pub mod retrieve_swarm_nodes;
mod rpc;
mod store;

pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub use rpc::*;
