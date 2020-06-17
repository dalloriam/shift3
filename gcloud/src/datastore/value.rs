use google_datastore1::{ArrayValue, Value};

use crate::datastore::{DSEntity, ToEntity};

/// Wrapper around supported datastore-serializable types.
///
/// Used mainly by derive(ToEntity)
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum DatastoreValue {
    /// String datastore value.
    Str(String),

    /// Datastore value for IDs (u64).
    Id(u64),

    /// Int datastore value.
    Int(i32),

    /// Array datastore value.
    Array(Vec<DatastoreValue>),

    /// Map of string and datastore value.
    Map(DSEntity),
}

impl From<String> for DatastoreValue {
    fn from(v: String) -> Self {
        DatastoreValue::Str(v)
    }
}

impl From<i32> for DatastoreValue {
    fn from(v: i32) -> Self {
        DatastoreValue::Int(v)
    }
}

impl From<u64> for DatastoreValue {
    fn from(v: u64) -> Self {
        DatastoreValue::Id(v)
    }
}

impl<T> From<Vec<T>> for DatastoreValue
where
    DatastoreValue: From<T>,
{
    fn from(v: Vec<T>) -> Self {
        DatastoreValue::Array(v.into_iter().map(DatastoreValue::from).collect())
    }
}

impl<T> From<T> for DatastoreValue
where
    T: ToEntity,
{
    fn from(v: T) -> Self {
        DatastoreValue::Map(v.into_entity())
    }
}

impl From<DatastoreValue> for Value {
    fn from(v: DatastoreValue) -> Value {
        match v {
            DatastoreValue::Int(v) => Value {
                integer_value: Some(format!("{}", v)),
                ..Default::default()
            },
            DatastoreValue::Id(v) => Value {
                integer_value: Some(format!("{}", v)),
                ..Default::default()
            },
            DatastoreValue::Str(s) => Value {
                string_value: Some(s),
                ..Default::default()
            },
            DatastoreValue::Array(arr) => Value {
                array_value: Some(ArrayValue {
                    values: Some(arr.into_iter().map(|v| v.into()).collect()),
                }),
                ..Default::default()
            },
            DatastoreValue::Map(entity) => Value {
                entity_value: Some(entity.into()),
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{DSEntity, DatastoreValue, ToEntity};
    use crate as gcloud; // Hack for the derive macro.

    #[test]
    fn from_string() {
        let dv = DatastoreValue::from(String::from("asdf"));
        assert_eq!(dv, DatastoreValue::Str(String::from("asdf")))
    }

    #[test]
    fn from_int() {
        let dv = DatastoreValue::from(18);
        assert_eq!(dv, DatastoreValue::Int(18));
    }

    #[test]
    fn from_u64() {
        let dv = DatastoreValue::from(18 as u64);
        assert_eq!(dv, DatastoreValue::Id(18));
    }

    #[test]
    fn from_vec() {
        let dv = DatastoreValue::from(vec![1, 2, 3]);
        assert_eq!(
            dv,
            DatastoreValue::Array(vec![
                DatastoreValue::Int(1),
                DatastoreValue::Int(2),
                DatastoreValue::Int(3)
            ])
        );
    }

    #[test]
    fn from_sub_entity() {
        #[derive(ToEntity)]
        struct Person {
            age: i32,
        }

        let dv = DatastoreValue::from(Person { age: 42 });
        let mut expected_hashmap = HashMap::new();
        expected_hashmap.insert(String::from("age"), DatastoreValue::Int(42));

        assert_eq!(
            dv,
            DatastoreValue::Map(DSEntity {
                entity_id: "Person",
                entity_data: expected_hashmap
            })
        )
    }
}
