use std::collections::HashMap;
use std::iter::FromIterator;

use google_datastore1::{Entity, Key, PathElement};

use crate::datastore::DatastoreValue;

/// Thin wrapper around a google cloud datastore entity.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DSEntity {
    /// The Kind of the entity.
    pub entity_id: &'static str,

    /// The data fields of the entity.
    pub entity_data: HashMap<String, DatastoreValue>,
}

impl From<DSEntity> for Entity {
    fn from(ent: DSEntity) -> Entity {
        Entity {
            key: Some(Key {
                path: Some(vec![PathElement {
                    kind: Some(String::from(ent.entity_id)),
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
pub trait ToEntity {
    /// Returns an entity.
    fn into_entity(self) -> DSEntity;
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
            entity_id: "Dog",
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
