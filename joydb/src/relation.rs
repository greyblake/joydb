use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{JoydbError, Model};

#[derive(Debug)]
pub struct Relation<M: Model> {
    // We ignore meta while serializing and deserializing.
    pub(crate) meta: RelationMeta,

    // The actual records in the relation.
    pub(crate) records: Vec<M>,
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
    pub fn new() -> Self {
        Relation {
            meta: RelationMeta::default(),
            records: Vec::new(),
        }
    }

    pub fn new_with_records(records: Vec<M>) -> Self {
        Relation {
            meta: RelationMeta::default(),
            records,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.meta.is_dirty
    }

    pub fn reset_dirty(&mut self) {
        self.meta.is_dirty = false;
    }

    /// Returns reference to the records.
    /// This is intended to be used only by partitioned adapters.
    pub fn records(&self) -> &[M] {
        &self.records
    }

    pub(crate) fn insert(&mut self, record: &M) -> Result<(), JoydbError> {
        let id = record.id();
        let is_duplicated = self.records.iter().any(|m| m.id() == id);
        if is_duplicated {
            Err(JoydbError::DuplicatedId {
                id: format!("{:?}", id),
                model_name: M::relation_name().to_owned(),
            })
        } else {
            self.records.push(record.clone());
            self.meta.is_dirty = true;
            Ok(())
        }
    }

    pub(crate) fn get(&self, id: &M::Id) -> Result<Option<M>, JoydbError> {
        let maybe_record = self.records.iter().find(|m| m.id() == id).cloned();
        Ok(maybe_record)
    }

    pub(crate) fn get_all(&self) -> Result<Vec<M>, JoydbError> {
        Ok(self.records.to_vec())
    }

    /// Return all records that match the predicate.
    pub(crate) fn get_all_by<F>(&self, predicate: F) -> Result<Vec<M>, JoydbError>
    where
        F: Fn(&M) -> bool,
    {
        let filtered_records = self
            .records
            .iter()
            .filter(|m| predicate(m))
            .cloned()
            .collect();
        Ok(filtered_records)
    }

    pub(crate) fn count(&self) -> Result<usize, JoydbError> {
        Ok(self.records.len())
    }

    // TODO: pass reference, must be consistent with `insert()`
    pub(crate) fn update(&mut self, new_record: M) -> Result<(), JoydbError> {
        let id = new_record.id();

        if let Some(m) = self.records.iter_mut().find(|m| m.id() == id) {
            *m = new_record;
            self.meta.is_dirty = true;
            Ok(())
        } else {
            Err(JoydbError::NotFound {
                id: format!("{:?}", id),
                model_name: M::relation_name().to_owned(),
            })
        }
    }

    pub(crate) fn upsert(&mut self, record: &M) -> Result<(), JoydbError> {
        let target_id = record.id();
        let maybe_target_record = self.records.iter_mut().find(|m| m.id() == target_id);
        if let Some(target_record) = maybe_target_record {
            *target_record = record.clone();
        } else {
            self.records.push(record.clone());
        }
        self.meta.is_dirty = true;
        Ok(())
    }

    pub(crate) fn delete(&mut self, id: &M::Id) -> Result<Option<M>, JoydbError> {
        let index = self.records.iter().position(|m| m.id() == id);
        if let Some(index) = index {
            let record = self.records.remove(index);
            self.meta.is_dirty = true;
            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn delete_all_by<F>(&mut self, predicate: F) -> Result<Vec<M>, JoydbError>
    where
        F: Fn(&M) -> bool,
    {
        let mut deleted_records = Vec::new();
        let mut retained_records = Vec::with_capacity(self.records.len());

        for record in self.records.drain(..) {
            if predicate(&record) {
                deleted_records.push(record);
                self.meta.is_dirty = true;
            } else {
                retained_records.push(record);
            }
        }
        self.records = retained_records;

        Ok(deleted_records)
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
        self.records.serialize(serializer)
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
            records: models,
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
        vec![first_post(), second_post(), third_post()]
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

    fn third_post() -> Post {
        Post {
            id: 3,
            title: "Third".to_string(),
        }
    }

    fn sample_relation() -> Relation<Post> {
        Relation {
            meta: RelationMeta { is_dirty: false },
            records: sample_posts(),
        }
    }

    mod serialization_and_deserialization {
        use super::*;

        #[test]
        fn test_serialize_relation() {
            let relation = Relation {
                meta: RelationMeta { is_dirty: false },
                records: sample_posts(),
            };

            let json = serde_json::to_string(&relation).unwrap();
            assert_eq!(
                json,
                r#"[{"id":1,"title":"First"},{"id":2,"title":"Second"},{"id":3,"title":"Third"}]"#
            );
        }

        #[test]
        fn test_deserialize_relation() {
            let json = r#"[{"id":10,"title":"One"},{"id":20,"title":"Two"}]"#;

            let relation: Relation<Post> = serde_json::from_str(json).unwrap();

            assert_eq!(relation.records.len(), 2);
            assert_eq!(relation.records[0].id, 10);
            assert_eq!(relation.records[0].title, "One");
            assert_eq!(relation.records[1].id, 20);
            assert_eq!(relation.records[1].title, "Two");

            // The meta field should be default-initialized
            assert_eq!(relation.meta.is_dirty, false);
        }

        #[test]
        fn test_serialize_deserialize_roundtrip() {
            let original = Relation {
                meta: RelationMeta { is_dirty: true },
                records: sample_posts(),
            };

            let json = serde_json::to_string(&original).unwrap();
            let deserialized: Relation<Post> = serde_json::from_str(&json).unwrap();

            assert_eq!(original.records, deserialized.records);
            assert_eq!(deserialized.meta.is_dirty, false); // Meta is not serialized
        }
    }

    mod insert {
        use super::*;

        #[test]
        fn should_insert_new_record_and_mark_dirty() {
            let mut relation = sample_relation();
            assert_eq!(relation.records.len(), 3);
            assert_eq!(relation.meta.is_dirty, false);

            let post = Post {
                id: 13,
                title: "Thirteen".to_string(),
            };
            relation.insert(&post).unwrap();

            assert_eq!(relation.records.len(), 4);
            assert_eq!(relation.records[3], post);
            assert_eq!(relation.meta.is_dirty, true);
        }

        #[test]
        fn should_return_an_error_when_record_with_id_already_exists() {
            let mut relation = Relation::new();
            let post = Post {
                id: 777,
                title: "First".to_string(),
            };
            relation.insert(&post).unwrap();

            let another_post = Post {
                id: 777,
                title: "Another First".to_string(),
            };
            let err = relation.insert(&another_post).unwrap_err();

            assert!(matches!(err, JoydbError::DuplicatedId { .. }));
            assert_eq!(
                err.to_string(),
                format!("Post with id = 777 already exists")
            );
        }
    }

    mod get {
        use super::*;

        #[test]
        fn should_return_none_when_record_not_found() {
            let relation = sample_relation();
            let id = 999;
            let maybe_post = relation.get(&id).unwrap();
            assert!(maybe_post.is_none());
        }

        #[test]
        fn should_return_record_when_found() {
            let relation = sample_relation();
            let id = 2;
            let maybe_post = relation.get(&id).unwrap();
            let post = maybe_post.unwrap();
            assert_eq!(post, second_post());
        }
    }

    mod get_all {
        use super::*;

        #[test]
        fn should_return_all_records() {
            let relation = sample_relation();
            let all_posts = relation.get_all().unwrap();
            assert_eq!(all_posts, sample_posts());
        }
    }

    mod count {
        use super::*;

        #[test]
        fn should_return_number_of_records() {
            let relation = sample_relation();
            let count = relation.count().unwrap();
            assert_eq!(count, 3);
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

            let updated_post = relation.get(&2).unwrap().unwrap();
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

            assert!(matches!(err, JoydbError::NotFound { .. }));
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

            assert_eq!(relation.records.len(), 2);
            assert_eq!(relation.records[0], second_post());
            assert_eq!(relation.meta.is_dirty, true);
            assert_eq!(deleted_post, first_post());
        }

        #[test]
        fn should_return_none_when_record_not_found() {
            let mut relation = sample_relation();
            let id = 555;
            let maybe_post = relation.delete(&id).unwrap();
            assert!(maybe_post.is_none());
            assert_eq!(relation.records.len(), 3);
            assert_eq!(relation.meta.is_dirty, false);
        }
    }

    mod delete_all_by {
        use super::*;

        #[test]
        fn should_delete_all_records_that_match_predicate() {
            let mut relation = sample_relation();
            assert_eq!(relation.records.len(), 3);

            let deleted_records = relation.delete_all_by(|post: &Post| post.id >= 2).unwrap();

            assert_eq!(deleted_records.len(), 2);
            assert!(deleted_records.contains(&second_post()));
            assert!(deleted_records.contains(&third_post()));

            assert_eq!(relation.records.len(), 1);
            assert_eq!(relation.records[0], first_post());
            assert_eq!(relation.meta.is_dirty, true);
        }

        #[test]
        fn should_not_delete_anything_if_no_record_matches_predicate() {
            let mut relation = sample_relation();
            assert_eq!(relation.records.len(), 3);

            let deleted_records = relation.delete_all_by(|post: &Post| post.id > 777).unwrap();

            assert_eq!(deleted_records.len(), 0);

            assert_eq!(relation.records.len(), 3);
            assert_eq!(relation.meta.is_dirty, false);
        }
    }

    mod upsert {
        use super::*;

        #[test]
        fn should_add_new_record_if_does_not_exist_yet() {
            let mut relation = sample_relation();
            assert_eq!(relation.records.len(), 3);

            let post44 = Post {
                id: 44,
                title: "Forty Four".to_string(),
            };
            relation.upsert(&post44).unwrap();

            assert_eq!(relation.records.len(), 4);
            assert_eq!(relation.records[3], post44);
        }

        #[test]
        fn should_update_existing_record_matched_by_id() {
            let mut relation = sample_relation();
            assert_eq!(relation.records.len(), 3);

            let updated_post2 = Post {
                id: 2,
                title: "Updated Second!!!".to_string(),
            };

            relation.upsert(&updated_post2).unwrap();

            assert_eq!(relation.records.len(), 3);
            assert_eq!(relation.get(&2).unwrap(), Some(updated_post2));
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
