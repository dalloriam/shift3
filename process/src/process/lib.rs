mod config;
mod node;
mod resource_manager;

use resource_manager::ResourceManager;

pub use config::Configuration;
pub use node::Node;

type Service = Box<dyn toolkit::Stop<Error = anyhow::Error> + Send>;
