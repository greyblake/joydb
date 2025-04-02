use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ToydbError;
use crate::traits::Model;

#[derive(Debug)]
pub struct Relation<M: Model> {
    // We ignore meta while serializing and deserializing.
    pub(crate) meta: RelationMeta,

    // The actual models in the relation.
    pub(crate) models: Vec<M>,
}

impl<M> Default for Relation<M>
where
    M: Model,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<M: Model> Relation<M> {
    pub fn is_dirty(&self) -> bool {
        self.meta.is_dirty
    }

    pub fn reset_dirty(&mut self) {
        self.meta.is_dirty = false;
    }

    pub(crate) fn new() -> Self {
        Relation {
            meta: RelationMeta::default(),
            models: Vec::new(),
        }
    }

    // TODO:
    // - Change to `model: &M`, don't take ownership
    // - Don't return the model, return `()`.
    pub(crate) fn insert(&mut self, model: M) -> Result<M, ToydbError> {
        let id = model.id();
        let is_duplicated = self.models.iter().find(|m| m.id() == id).is_some();
        if is_duplicated {
            return Err(ToydbError::DuplicatedId {
                id: format!("{:?}", id),
                model_name: M::relation_name().to_owned(),
            });
        } else {
            self.models.push(model.clone());
            self.meta.is_dirty = true;
            Ok(model)
        }
    }

    pub(crate) fn find(&self, id: &M::Id) -> Result<Option<M>, ToydbError> {
        let maybe_record = self.models.iter().find(|m| m.id() == id).cloned();
        Ok(maybe_record)
    }

    pub(crate) fn all(&self) -> Result<Vec<M>, ToydbError> {
        Ok(self.models.to_vec())
    }

    pub(crate) fn count(&self) -> Result<usize, ToydbError> {
        Ok(self.models.len())
    }

    pub(crate) fn update(&mut self, new_model: M) -> Result<(), ToydbError> {
        let id = new_model.id();

        if let Some(m) = self.models.iter_mut().find(|m| m.id() == id) {
            *m = new_model;
            self.meta.is_dirty = true;
            Ok(())
        } else {
            Err(ToydbError::NotFound {
                id: format!("{:?}", id),
                model_name: M::relation_name().to_owned(),
            })
        }
    }

    pub(crate) fn delete(&mut self, id: &M::Id) -> Result<Option<M>, ToydbError> {
        let index = self.models.iter().position(|m| m.id() == id);
        if let Some(index) = index {
            let record = self.models.remove(index);
            self.meta.is_dirty = true;
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }
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
        vec![first_post(), second_post()]
    }

    fn first_post() -> Post {
        Post {
            id: 1,
            title: "First".to_string(),
        }
    }

    fn second_post() -> Post {
        Post {
            id: 2,
            title: "Second".to_string(),
        }
    }

    fn sample_relation() -> Relation<Post> {
        Relation {
            meta: RelationMeta { is_dirty: false },
            models: sample_posts(),
        }
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

    mod insert {
        use super::*;

        #[test]
        fn should_insert_new_record_and_mark_dirty() {
            let mut relation = sample_relation();
            assert_eq!(relation.models.len(), 2);
            assert_eq!(relation.meta.is_dirty, false);

            let post = Post {
                id: 13,
                title: "Thirteen".to_string(),
            };
            relation.insert(post.clone()).unwrap();

            assert_eq!(relation.models.len(), 3);
            assert_eq!(relation.models[2], post);
            assert_eq!(relation.meta.is_dirty, true);
        }

        #[test]
        fn should_return_an_error_when_record_with_id_already_exists() {
            let mut relation = Relation::new();
            let post = Post {
                id: 777,
                title: "First".to_string(),
            };
            relation.insert(post.clone()).unwrap();

            let another_post = Post {
                id: 777,
                title: "Another First".to_string(),
            };
            let err = relation.insert(another_post.clone()).unwrap_err();

            assert!(matches!(err, ToydbError::DuplicatedId { .. }));
            assert_eq!(
                err.to_string(),
                format!("Post with id = 777 already exists")
            );
        }
    }

    mod find {
        use super::*;

        #[test]
        fn should_return_none_when_record_not_found() {
            let relation = sample_relation();
            let id = 999;
            let maybe_post = relation.find(&id).unwrap();
            assert!(maybe_post.is_none());
        }

        #[test]
        fn should_return_record_when_found() {
            let relation = sample_relation();
            let id = 2;
            let maybe_post = relation.find(&id).unwrap();
            let post = maybe_post.unwrap();
            assert_eq!(post, second_post());
        }
    }

    mod all {
        use super::*;

        #[test]
        fn should_return_all_records() {
            let relation = sample_relation();
            let all_posts = relation.all().unwrap();
            assert_eq!(all_posts, sample_posts());
        }
    }

    mod count {
        use super::*;

        #[test]
        fn should_return_number_of_records() {
            let relation = sample_relation();
            let count = relation.count().unwrap();
            assert_eq!(count, 2);
        }
    }

    mod update {
        use super::*;

        #[test]
        fn should_update_record_and_mark_dirty() {
            let mut relation = sample_relation();
            let new_post = Post {
                id: 2,
                title: "Updated Second".to_string(),
            };
            relation.update(new_post.clone()).unwrap();

            let updated_post = relation.find(&2).unwrap().unwrap();
            assert_eq!(updated_post, new_post);
            assert_eq!(relation.meta.is_dirty, true);
        }

        #[test]
        fn should_return_error_when_record_not_found() {
            let mut relation = sample_relation();
            let new_post = Post {
                id: 999,
                title: "Updated Second".to_string(),
            };
            let err = relation.update(new_post).unwrap_err();

            assert!(matches!(err, ToydbError::NotFound { .. }));
            assert_eq!(err.to_string(), format!("Post with id = 999 not found"));
        }
    }

    mod delete {
        use super::*;

        #[test]
        fn should_delete_record_and_mark_dirty() {
            let mut relation = sample_relation();
            let id = 1;
            let deleted_post = relation.delete(&id).unwrap().unwrap();

            assert_eq!(relation.models.len(), 1);
            assert_eq!(relation.models[0], second_post());
            assert_eq!(relation.meta.is_dirty, true);
            assert_eq!(deleted_post, first_post());
        }

        #[test]
        fn should_return_none_when_record_not_found() {
            let mut relation = sample_relation();
            let id = 555;
            let maybe_post = relation.delete(&id).unwrap();
            assert!(maybe_post.is_none());
            assert_eq!(relation.models.len(), 2);
            assert_eq!(relation.meta.is_dirty, false);
        }
    }

    #[test]
    fn should_reset_dirty() {
        let mut relation = sample_relation();
        assert_eq!(relation.is_dirty(), false);

        relation.delete(&1).unwrap();
        assert_eq!(relation.is_dirty(), true);

        relation.reset_dirty();
        assert_eq!(relation.is_dirty(), false);
    }
}
