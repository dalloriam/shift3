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
    DeserializeItem { source: serde_json::Error },
    Flush { source: sled::Error },
    InsertItem { source: sled::Error },
    OpenTree { source: sled::Error },
    OpenDatabase { source: sled::Error },
    ReadItem { source: sled::Error },
    SerializeItem { source: serde_json::Error },
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
        let db = sled::open(persist_path.as_ref()).context(OpenDatabaseSnafu)?;
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
        let tree = db.open_tree(kind).context(OpenTreeSnafu)?;

        Ok(EntityStore {
            tree,
            _phantom: Default::default(),
        })
    }

    /// Flush the store to disk.
    pub fn flush(&self) -> Result<()> {
        self.tree.flush().context(FlushSnafu)?;
        Ok(())
    }

    /// Get an entity from this store by its ID.
    pub fn get(&self, id: &str) -> Result<Option<T>> {
        match self.tree.get(id).context(ReadItemSnafu)? {
            Some(item_ivec) => {
                let item =
                    serde_json::from_slice(item_ivec.as_ref()).context(DeserializeItemSnafu)?;
                Ok(Some(item))
            }
            None => Ok(None),
        }
    }

    /// Insert a new entity to this store.
    pub fn insert(&self, entity: &T) -> Result<String> {
        // Serialize the entity.
        let serialized_bytes = serde_json::to_vec(entity).context(SerializeItemSnafu)?;

        // Hash the serialization to get an ID.
        //  (This is the worst way to get an ID - impossible to update)
        //  TODO: Find a better way to handle this.
        let mut hasher = Sha1::new();
        hasher.update(&serialized_bytes);
        let result = hasher.finalize();

        let id_str = format!("{:x}", result);

        self.tree
            .insert(id_str.clone(), serialized_bytes)
            .context(InsertItemSnafu)?;

        Ok(id_str)
    }

    /// Lists all entities of this store's type.
    pub fn list_all(&self) -> Result<Vec<T>> {
        // TODO: Paging.

        let mut results: Vec<T> = Vec::new();
        for tuple_maybe in self.tree.iter() {
            let (_key_ivec, val_ivec) = tuple_maybe.context(ReadItemSnafu)?;
            let val = serde_json::from_slice(val_ivec.as_ref()).context(DeserializeItemSnafu)?;
            results.push(val);
        }

        Ok(results)
    }
}
