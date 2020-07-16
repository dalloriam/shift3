use std::collections::HashMap;
use std::convert::TryFrom;
use std::iter::FromIterator;

use google_datastore1::{Entity, Key, PathElement};

use snafu::{ensure, Snafu};

use crate::datastore::DatastoreValue;

#[derive(Debug, Snafu)]
pub enum Error {
    InvalidEntityData,
}

/// Thin wrapper around a google cloud datastore entity.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DSEntity {
    /// The name of the entity.
    pub entity_name: Option<String>,

    /// The Kind of the entity.
    pub entity_kind: String,

    /// The data fields of the entity.
    pub entity_data: HashMap<String, DatastoreValue>,
}

impl TryFrom<Entity> for DSEntity {
    type Error = Error;

    fn try_from(ent: Entity) -> Result<DSEntity, Error> {
        let entity_key = ent.key.ok_or(Error::InvalidEntityData)?;
        let mut path = entity_key.path.ok_or(Error::InvalidEntityData)?;

        ensure!(!path.is_empty(), InvalidEntityData);
        let root_elem: PathElement = path.remove(0); // Panics if vec is empty -- hence ensure!()

        let entity_kind = root_elem.kind.ok_or(Error::InvalidEntityData)?;
        let entity_name = root_elem.name;

        let properties = ent.properties.ok_or(Error::InvalidEntityData)?;

        let mut entity_data = HashMap::new();

        for (k, v) in properties.into_iter() {
            entity_data.insert(
                k,
                DatastoreValue::try_from(v).map_err(|_e| Error::InvalidEntityData)?,
            );
        }

        Ok(DSEntity {
            entity_name,
            entity_kind,
            entity_data,
        })
    }
}

impl From<DSEntity> for Entity {
    fn from(ent: DSEntity) -> Entity {
        Entity {
            key: Some(Key {
                path: Some(vec![PathElement {
                    kind: Some(ent.entity_kind),
                    name: ent.entity_name,
                    ..Default::default()
                }]),
                partition_id: None,
            }),
            properties: Some(HashMap::from_iter(
                ent.entity_data
                    .into_iter()
                    .map(|(key, val)| (key, val.into())),
            )),
        }
    }
}

/// Converts a type to a metadata map format supported by the
/// Google Datastore API.
pub trait DatastoreEntity {
    /// Returns an entity.
    fn into_entity(self) -> DSEntity;

    /// Return the entity kind.
    fn get_kind() -> &'static str;

    /// Rebuild the original type from a DS entity.
    fn from_entity(ds: DSEntity) -> Self;
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use google_datastore1::Entity;

    use super::{DSEntity, DatastoreValue};

    #[test]
    fn entity_from_ds() {
        // Setup test entity.
        let mut entity_data = HashMap::new();
        entity_data.insert(
            String::from("breed"),
            DatastoreValue::Str(String::from("Husky")),
        );
        let ds = DSEntity {
            entity_name: None,
            entity_kind: String::from("Dog"),
            entity_data,
        };

        let converted = Entity::from(ds);

        // !*&#*!@&# google cloud library that doesn't implement PartialEq on its types.
        let serialized_entity = serde_json::to_string(&converted).unwrap();
        assert!(serialized_entity.contains("breed"));
        assert!(serialized_entity.contains("Husky"));
        assert!(serialized_entity.contains("Dog"));
    }
}
