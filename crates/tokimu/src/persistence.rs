use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;

pub type PersistenceResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub trait DocumentCodec {
    fn save<T>(&self, document: &T) -> PersistenceResult<String>
    where
        T: Serialize;

    fn load<T>(&self, source: &str) -> PersistenceResult<T>
    where
        T: DeserializeOwned;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RonDocumentCodec;

impl DocumentCodec for RonDocumentCodec {
    fn save<T>(&self, document: &T) -> PersistenceResult<String>
    where
        T: Serialize,
    {
        ron::ser::to_string_pretty(document, ron::ser::PrettyConfig::default())
            .map_err(Into::into)
    }

    fn load<T>(&self, source: &str) -> PersistenceResult<T>
    where
        T: DeserializeOwned,
    {
        ron::de::from_str(source).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct SavedToyState {
        score: u32,
        title: String,
    }

    #[test]
    fn ron_document_codec_round_trips_a_resource() {
        let codec = RonDocumentCodec;
        let original = SavedToyState {
            score: 42,
            title: "hello-triangle".into(),
        };

        let encoded = codec.save(&original).expect("document should encode");
        let decoded: SavedToyState = codec.load(&encoded).expect("document should decode");

        assert_eq!(decoded, original);
        assert!(encoded.contains("score"));
    }
}