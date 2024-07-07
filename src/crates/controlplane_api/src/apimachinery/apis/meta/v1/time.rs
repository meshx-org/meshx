/// Time is a wrapper around time.Time which supports correct marshaling to YAML and JSON.  Wrappers are provided for many of the factory methods that the time package offers.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Time(pub chrono::DateTime<chrono::Utc>);

impl<'de> serde::Deserialize<'de> for Time {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Time;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("Time")
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Ok(Time(serde::Deserialize::deserialize(deserializer)?))
            }
        }

        deserializer.deserialize_newtype_struct("Time", Visitor)
    }
}

impl serde::Serialize for Time {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct("Time", &self.0.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
    }
}

impl schemars::JsonSchema for Time {
    fn schema_name() -> String {
        "co.meshx.apimachinery.apis.meta.v1.Time".to_owned()
    }

    fn json_schema(__gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        schemars::schema::Schema::Object(schemars::schema::SchemaObject {
            metadata: Some(Box::new(schemars::schema::Metadata {
                description: Some("Time is a wrapper around time.Time which supports correct marshaling to YAML and JSON.  Wrappers are provided for many of the factory methods that the time package offers.".to_owned()),
                ..Default::default()
            })),
            instance_type: Some(schemars::schema::SingleOrVec::Single(Box::new(schemars::schema::InstanceType::String))),
            format: Some("date-time".to_owned()),
            ..Default::default()
        })
    }
}
