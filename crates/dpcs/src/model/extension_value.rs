//! Format-neutral extension values for the Canonical Object Model.
//!
//! Extension storage is independent of any particular serialization format.
//! Serde conversions to and from `serde_json::Value` exist for JSON interoperability;
//! deserialization uses a format-neutral visitor so YAML documents do not need an
//! intermediate JSON value.

use std::fmt;

use indexmap::IndexMap;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Number;

/// A format-neutral extension value.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtensionValue {
    /// Null value.
    Null,
    /// Boolean value.
    Bool(bool),
    /// Numeric value.
    Number(Number),
    /// String value.
    String(String),
    /// Array of extension values.
    Array(Vec<ExtensionValue>),
    /// Object map of extension values.
    Object(ExtensionMap),
}

/// A map of extension field names to extension values.
pub type ExtensionMap = IndexMap<String, ExtensionValue>;

#[cfg(feature = "jsonschema")]
impl schemars::JsonSchema for ExtensionValue {
    fn schema_name() -> String {
        "ExtensionValue".to_owned()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        // Extensions are arbitrary JSON-compatible values.
        schemars::schema::Schema::Bool(true)
    }
}

impl ExtensionValue {
    /// Returns `true` when this value is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Returns the string value when this value is a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    /// Returns a nested value by key when this value is an object.
    pub fn get(&self, key: &str) -> Option<&ExtensionValue> {
        self.as_object().and_then(|map| map.get(key))
    }

    /// Returns the object map when this value is an object.
    pub fn as_object(&self) -> Option<&ExtensionMap> {
        match self {
            Self::Object(map) => Some(map),
            _ => None,
        }
    }
}

impl fmt::Display for ExtensionValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::Number(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "{value}"),
            Self::Array(values) => write!(f, "{values:?}"),
            Self::Object(map) => write!(f, "{map:?}"),
        }
    }
}

impl Serialize for ExtensionValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Null => serializer.serialize_unit(),
            Self::Bool(value) => serializer.serialize_bool(*value),
            Self::Number(value) => serde_json::Value::Number(value.clone()).serialize(serializer),
            Self::String(value) => serializer.serialize_str(value),
            Self::Array(values) => values.serialize(serializer),
            Self::Object(map) => map.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for ExtensionValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ExtensionValueVisitor)
    }
}

struct ExtensionValueVisitor;

impl<'de> Visitor<'de> for ExtensionValueVisitor {
    type Value = ExtensionValue;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a JSON-compatible extension value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(ExtensionValue::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(ExtensionValue::Number(Number::from(value)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(ExtensionValue::Number(Number::from(value)))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if let Some(number) = Number::from_f64(value) {
            Ok(ExtensionValue::Number(number))
        } else {
            // Non-finite YAML numbers are not JSON-compatible; preserve as strings.
            Ok(ExtensionValue::String(value.to_string()))
        }
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(ExtensionValue::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(ExtensionValue::String(value))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(ExtensionValue::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(ExtensionValue::Null)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element()? {
            values.push(value);
        }
        Ok(ExtensionValue::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut object = ExtensionMap::new();
        while let Some((key, value)) = map.next_entry()? {
            object.insert(key, value);
        }
        Ok(ExtensionValue::Object(object))
    }
}

impl From<serde_json::Value> for ExtensionValue {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(value) => Self::Bool(value),
            serde_json::Value::Number(value) => Self::Number(value),
            serde_json::Value::String(value) => Self::String(value),
            serde_json::Value::Array(values) => {
                Self::Array(values.into_iter().map(Self::from).collect())
            }
            serde_json::Value::Object(map) => Self::Object(
                map.into_iter()
                    .map(|(key, value)| (key, Self::from(value)))
                    .collect(),
            ),
        }
    }
}

impl From<ExtensionValue> for serde_json::Value {
    fn from(value: ExtensionValue) -> Self {
        match value {
            ExtensionValue::Null => Self::Null,
            ExtensionValue::Bool(value) => Self::Bool(value),
            ExtensionValue::Number(value) => Self::Number(value),
            ExtensionValue::String(value) => Self::String(value),
            ExtensionValue::Array(values) => {
                Self::Array(values.into_iter().map(Self::from).collect())
            }
            ExtensionValue::Object(map) => Self::Object(
                map.into_iter()
                    .map(|(key, value)| (key, Self::from(value)))
                    .collect(),
            ),
        }
    }
}

impl PartialEq<str> for ExtensionValue {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == Some(other)
    }
}

impl PartialEq<&str> for ExtensionValue {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == Some(*other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_serde_json() {
        let json = serde_json::json!({
            "team": "data-platform",
            "count": 3,
            "enabled": true,
            "nested": { "key": "value" }
        });

        let extension: ExtensionValue = json.clone().into();
        let back: serde_json::Value = extension.into();
        assert_eq!(json, back);
    }

    #[test]
    fn yaml_null_and_non_finite_extensions_do_not_fail_parse() {
        let value: ExtensionValue = serde_yaml::from_str("null").unwrap();
        assert!(value.is_null());

        let nan: ExtensionValue = serde_yaml::from_str(".nan").unwrap();
        assert_eq!(nan.as_str(), Some("NaN"));
    }
}
