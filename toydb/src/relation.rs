use crate::traits::Model;

use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

pub struct Relation<M: Model> {
    // We ignore meta while serializing and deserializing.
    pub(crate) meta: RelationMeta,

    // The actual models in the relation.
    pub(crate) models: Vec<M>,
}

impl<M: Model> Relation<M> {}

/// Metadata for the relation.
/// It's not serialized or persisted. They meant to exist only in memory.
#[derive(Debug, Default)]
pub struct RelationMeta {
    pub(crate) is_dirty: bool,
}

// Custom serialization for Relation
impl<M: Model> Serialize for Relation<M> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.models.serialize(serializer)
    }
}

// Custom deserialization for Relation
impl<'de, M: Model> Deserialize<'de> for Relation<M> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let models = Vec::<M>::deserialize(deserializer)?;
        Ok(Relation {
            meta: RelationMeta::default(),
            models,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct Post {
        id: u32,
        title: String,
    }

    impl Model for Post {
        type Id = u32;

        fn id(&self) -> &Self::Id {
            &self.id
        }
    }

    fn sample_posts() -> Vec<Post> {
        vec![
            Post {
                id: 1,
                title: "First".to_string(),
            },
            Post {
                id: 2,
                title: "Second".to_string(),
            },
        ]
    }

    mod serialization_and_deserialization {
        use super::*;

        #[test]
        fn test_serialize_relation() {
            let relation = Relation {
                meta: RelationMeta { is_dirty: false },
                models: sample_posts(),
            };

            let json = serde_json::to_string(&relation).unwrap();
            assert_eq!(
                json,
                r#"[{"id":1,"title":"First"},{"id":2,"title":"Second"}]"#
            );
        }

        #[test]
        fn test_deserialize_relation() {
            let json = r#"[{"id":10,"title":"One"},{"id":20,"title":"Two"}]"#;

            let relation: Relation<Post> = serde_json::from_str(json).unwrap();

            assert_eq!(relation.models.len(), 2);
            assert_eq!(relation.models[0].id, 10);
            assert_eq!(relation.models[0].title, "One");
            assert_eq!(relation.models[1].id, 20);
            assert_eq!(relation.models[1].title, "Two");

            // The meta field should be default-initialized
            assert_eq!(relation.meta.is_dirty, false);
        }

        #[test]
        fn test_serialize_deserialize_roundtrip() {
            let original = Relation {
                meta: RelationMeta { is_dirty: true },
                models: sample_posts(),
            };

            let json = serde_json::to_string(&original).unwrap();
            let deserialized: Relation<Post> = serde_json::from_str(&json).unwrap();

            assert_eq!(original.models, deserialized.models);
            assert_eq!(deserialized.meta.is_dirty, false); // Meta is not serialized
        }
    }
}
