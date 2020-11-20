//! Sled-based entity store.

use std::marker;
use std::path::Path;

use serde::{de::DeserializeOwned, Serialize};

use sha1::{Digest, Sha1};

use snafu::{ResultExt, Snafu};

// TODO: Use bincode instead of JSON.

#[allow(missing_docs)]
#[derive(Debug, Snafu)]
pub enum Error {
    FailedToDeserializeItem { source: serde_json::Error },
    FailedToFlush { source: sled::Error },
    FailedToInsertItem { source: sled::Error },
    FailedToOpenTree { source: sled::Error },
    FailedToOpenDatabase { source: sled::Error },
    FailedToReadItem { source: sled::Error },
    FailedToSerializeItem { source: serde_json::Error },
}

type Result<T> = std::result::Result<T, Error>;

/// Handle to a sled database.
pub struct SledStore {
    db: sled::Db,
}

// TODO: Add Doctests.
impl SledStore {
    /// Create or open a sled database at the provided directory.
    pub fn new<P: AsRef<Path>>(persist_path: P) -> Result<Self> {
        let db = sled::open(persist_path.as_ref()).context(FailedToOpenDatabase)?;
        Ok(SledStore { db })
    }

    /// Get a handle to subset of the database containing only entities of this type.
    pub fn entity<T>(&self, kind: &str) -> Result<EntityStore<T>>
    where
        T: Serialize + DeserializeOwned,
    {
        EntityStore::new(&self.db, kind)
    }
}

/// A handle to a subset of the sled tree.
pub struct EntityStore<T>
where
    T: Serialize + DeserializeOwned,
{
    tree: sled::Tree,

    _phantom: marker::PhantomData<T>,
}

impl<T> EntityStore<T>
where
    T: Serialize + DeserializeOwned,
{
    /// Fetch a subtree from the database and return a new handle to it.
    pub fn new(db: &sled::Db, kind: &str) -> Result<EntityStore<T>> {
        let tree = db.open_tree(kind).context(FailedToOpenTree)?;

        Ok(EntityStore {
            tree,
            _phantom: Default::default(),
        })
    }

    pub fn flush(&self) -> Result<()> {
        self.tree.flush().context(FailedToFlush)?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Option<T>> {
        match self.tree.get(id).context(FailedToReadItem)? {
            Some(item_ivec) => {
                let item =
                    serde_json::from_slice(item_ivec.as_ref()).context(FailedToDeserializeItem)?;
                Ok(Some(item))
            }
            None => Ok(None),
        }
    }

    pub fn insert(&self, entity: &T) -> Result<String> {
        // Serialize the entity.
        let serialized_bytes = serde_json::to_vec(entity).context(FailedToSerializeItem)?;

        // Hash the serialization to get an ID.
        //  (This is the worst way to get an ID - impossible to update)
        //  TODO: Find a better way to handle this.
        let mut hasher = Sha1::new();
        hasher.update(&serialized_bytes);
        let result = hasher.finalize();

        let id_str = format!("{:x}", result);

        self.tree
            .insert(id_str.clone(), serialized_bytes)
            .context(FailedToInsertItem)?;

        Ok(id_str)
    }

    pub fn list_all(&self) -> Result<Vec<T>> {
        // TODO: Paging.

        let mut results: Vec<T> = Vec::new();
        for tuple_maybe in self.tree.iter() {
            let (_key_ivec, val_ivec) = tuple_maybe.context(FailedToReadItem)?;
            let val = serde_json::from_slice(val_ivec.as_ref()).context(FailedToDeserializeItem)?;
            results.push(val);
        }

        Ok(results)
    }
}
