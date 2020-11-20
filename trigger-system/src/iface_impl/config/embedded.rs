use std::sync::Arc;

use anyhow::Result;

use async_trait::async_trait;

use toolkit::db::sled::{EntityStore, SledStore};

use crate::interface::{TriggerConfigLoader, TriggerConfiguration};

pub struct EmbeddedTriggerConfigLoader {
    store: EntityStore<TriggerConfiguration>,
}

impl EmbeddedTriggerConfigLoader {
    pub fn new(db: Arc<SledStore>) -> Result<Self> {
        let store = db.entity("TriggerConfiguration")?;
        Ok(Self { store })
    }
}

#[async_trait]
impl TriggerConfigLoader for EmbeddedTriggerConfigLoader {
    async fn get_all_configurations(&self) -> Result<Vec<TriggerConfiguration>> {
        let entities = self.store.list_all()?;
        Ok(entities)
    }
}
