use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ToydbError;
use crate::traits::Model;

pub struct Relation<M: Model> {
    // We ignore meta while serializing and deserializing.
    pub(crate) meta: RelationMeta,

    // The actual models in the relation.
    pub(crate) models: Vec<M>,
}

impl<M: Model> Relation<M> {
    /*

    fn insert(&mut self, model: M) -> Result<M, ToydbError> {
        let id = model.id();
        let is_duplicated = self.models
            .iter().find(|m| m.id() == id).is_some();
        if is_duplicated {
            return Err(ToydbError::DuplicatedId {
                id: format!("{:?}", id),
                model_name: base_type_name::<M>().to_owned(),
            });
        } else {
            relation.push(model.clone());
            self.is_dirty = true;
            Ok(model)
        }
    }
    */

    /*
    fn find<M: Model>(&self, id: &M::Id) -> Result<Option<M>, ToydbError>
    where
        State: GetRelation<M>,
    {
        let relation = self.get_relation::<M>();
        let maybe_record = relation.iter().find(|m| m.id() == id).cloned();
        Ok(maybe_record)
    }

    fn all<M: Model>(&self) -> Result<Vec<M>, ToydbError>
    where
        State: GetRelation<M>,
    {
        let records = self.get_relation::<M>().to_vec();
        Ok(records)
    }

    pub fn count<M: Model>(&self) -> Result<usize, ToydbError>
    where
        State: GetRelation<M>,
    {
        Ok(self.get_relation::<M>().len())
    }

    fn update<M: Model>(&mut self, new_model: M) -> Result<(), ToydbError>
    where
        State: GetRelation<M>,
    {
        let relation = self.get_relation_mut::<M>();

        let id = new_model.id();
        if let Some(m) = relation.iter_mut().find(|m| m.id() == id) {
            *m = new_model;
            self.is_dirty = true;
            Ok(())
        } else {
            Err(ToydbError::NotFound {
                id: format!("{:?}", id),
                model_name: base_type_name::<M>().to_owned(),
            })
        }
    }

    fn delete<M: Model>(&mut self, id: &M::Id) -> Result<Option<M>, ToydbError>
    where
        State: GetRelation<M>,
    {
        let relation = self.get_relation_mut::<M>();

        let index = relation.iter().position(|m| m.id() == id);
        if let Some(index) = index {
            let record = relation.remove(index);
            self.is_dirty = true;
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }
    */
}

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

        fn relation_name() -> &'static str {
            "Post"
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
