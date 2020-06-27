use std::convert::TryFrom;

use google_datastore1::{ArrayValue, Value};

use snafu::{ResultExt, Snafu};

use crate::datastore::EntityConversionError;

#[derive(Debug, Snafu)]
pub enum Error {
    DeserializationError,
    InvalidCast { source: std::num::ParseIntError },
    NestedEntityReadError { source: EntityConversionError },
}

type Result<T> = std::result::Result<T, Error>;

/// Wrapper around supported datastore-serializable types.
///
/// Used mainly by derive(DatastoreEntity)
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
}

impl From<String> for DatastoreValue {
    fn from(v: String) -> Self {
        DatastoreValue::Str(v)
    }
}

impl TryFrom<DatastoreValue> for String {
    type Error = Error;
    fn try_from(v: DatastoreValue) -> Result<String> {
        if let DatastoreValue::Str(s) = v {
            Ok(s)
        } else {
            Err(Error::DeserializationError)
        }
    }
}

impl From<i32> for DatastoreValue {
    fn from(v: i32) -> Self {
        DatastoreValue::Int(v)
    }
}

impl TryFrom<DatastoreValue> for i32 {
    type Error = Error;
    fn try_from(v: DatastoreValue) -> Result<i32> {
        if let DatastoreValue::Int(s) = v {
            Ok(s)
        } else {
            Err(Error::DeserializationError)
        }
    }
}

impl From<u64> for DatastoreValue {
    fn from(v: u64) -> Self {
        DatastoreValue::Id(v)
    }
}

impl TryFrom<DatastoreValue> for u64 {
    type Error = Error;
    fn try_from(v: DatastoreValue) -> Result<u64> {
        if let DatastoreValue::Id(s) = v {
            Ok(s)
        } else if let DatastoreValue::Int(i) = v {
            Ok(i as u64) // i32 -> u64 conversion is safe
        } else {
            Err(Error::DeserializationError)
        }
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

impl<T> TryFrom<DatastoreValue> for Vec<T>
where
    T: TryFrom<DatastoreValue>,
{
    type Error = Error;

    fn try_from(v: DatastoreValue) -> Result<Vec<T>> {
        if let DatastoreValue::Array(s) = v {
            let res: std::result::Result<Vec<T>, T::Error> =
                s.into_iter().map(T::try_from).collect();
            res.map_err(|_e| Error::DeserializationError)
        } else {
            Err(Error::DeserializationError)
        }
    }
}

impl TryFrom<Value> for DatastoreValue {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        if let Some(v) = value.integer_value {
            assert!(value.string_value.is_none());
            assert!(value.array_value.is_none());
            assert!(value.entity_value.is_none());

            // This is tricky because we encode both i32 & u64 in Value::integer_value.
            // This means that we have a bit of magic to do to read it back properly
            // without overflowing, underflowing, or truncating.

            // int128 can represent all values of u64 and i32.
            let i = v.parse::<i128>().context(InvalidCast)?;

            if i < 0 || i < std::u64::MAX as i128 {
                // Safe to convert to i32.
                Ok(DatastoreValue::Int(i as i32))
            } else {
                Ok(DatastoreValue::Id(i as u64))
            }
        } else if let Some(v) = value.string_value {
            assert!(value.array_value.is_none());
            assert!(value.entity_value.is_none());
            Ok(DatastoreValue::Str(v))
        } else if let Some(v) = value.array_value {
            assert!(value.entity_value.is_none());
            let values = v.values.ok_or(Error::DeserializationError)?;
            let converted_result: Result<Vec<DatastoreValue>> =
                values.into_iter().map(DatastoreValue::try_from).collect();
            Ok(DatastoreValue::Array(converted_result?))
        } else {
            Err(Error::DeserializationError)
        }
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DatastoreValue;

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
}
