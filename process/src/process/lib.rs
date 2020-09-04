mod config;
mod node;

pub use config::Configuration;
pub use node::Node;

type Service = Box<dyn toolkit::Stop<Error = anyhow::Error> + Send>;
