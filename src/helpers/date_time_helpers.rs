use chrono::{DateTime, Utc};
use mongodb::bson;
use serde::{Deserialize, Deserializer, Serializer};

pub mod optional_chrono_datetime {
    use super::*;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum DateTimeValue {
        Bson(bson::DateTime),
        String(String),
    }

    pub fn serialize<S>(value: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(date_time) if serializer.is_human_readable() => {
                serializer.serialize_some(&date_time.to_rfc3339())
            }
            Some(date_time) => {
                let bson_date_time = bson::DateTime::from_millis(date_time.timestamp_millis());
                serializer.serialize_some(&bson_date_time)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Option::<DateTimeValue>::deserialize(deserializer)?;

        value
            .map(|value| match value {
                DateTimeValue::Bson(date_time) => {
                    Ok(DateTime::<Utc>::from(date_time.to_system_time()))
                }
                DateTimeValue::String(date_time) => DateTime::parse_from_rfc3339(&date_time)
                    .map(|date_time| date_time.with_timezone(&Utc))
                    .map_err(serde::de::Error::custom),
            })
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::optional_chrono_datetime;
    use chrono::{DateTime, Utc};
    use mongodb::bson::{self, doc, Bson};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    struct TestDocument {
        #[serde(
            default,
            serialize_with = "optional_chrono_datetime::serialize",
            deserialize_with = "optional_chrono_datetime::deserialize"
        )]
        date_time: Option<DateTime<Utc>>,
    }

    #[test]
    fn deserializes_bson_datetime() {
        let parsed: TestDocument = bson::from_document(doc! {
            "date_time": bson::DateTime::now(),
        })
        .expect("BSON datetime should deserialize as chrono datetime");

        assert!(parsed.date_time.is_some());
    }

    #[test]
    fn serializes_as_bson_datetime_for_bson() {
        let document = TestDocument {
            date_time: Some(Utc::now()),
        };

        let serialized = bson::to_document(&document).expect("document should serialize");

        assert!(matches!(
            serialized.get("date_time"),
            Some(Bson::DateTime(_))
        ));
    }

    #[test]
    fn serializes_as_rfc3339_string_for_json() {
        let document = TestDocument {
            date_time: Some(Utc::now()),
        };

        let serialized = serde_json::to_value(&document).expect("json should serialize");

        assert!(serialized["date_time"].as_str().is_some());
    }
}
