use crate::Identifiable;

/// A relation (think of it as a table in RDBMS).
/// Provides simple methods for CRUD operations.
///
/// Panics:
/// * Inserting a record with ID, which already exists.
/// * Deleting a record by ID, which does not exist.
/// * Updating a record which does not exist.
///
/// At some points those panics may be replaced with returned errors.
pub struct Relation<'a, R: Identifiable + Clone> {
    records: &'a mut Vec<R>,
}

impl<'a, R> Relation<'a, R>
where
    R: Identifiable + Clone,
{
    pub fn new(records: &'a mut Vec<R>) -> Self {
        Self { records }
    }

    // Create
    //
    pub fn insert(&mut self, new_record: R) {
        let id = new_record.id();
        let found_record = self.records.iter().find(|r| r.id() == id);
        if found_record.is_some() {
            let typ = base_type_name::<R>();
            panic!("Cannot insert {typ} (id={id}), because a record with this ID already exists");
        }
        self.records.push(new_record);
    }

    pub fn insert_many(&mut self, new_records: Vec<R>) {
        for new_record in new_records {
            self.insert(new_record);
        }
    }

    // Read
    //
    pub fn get_all(&self) -> Vec<R> {
        self.records.clone()
    }

    pub fn get_by_id(&self, id: <R as Identifiable>::Id) -> Option<R> {
        self.records.iter().find(|r| r.id() == id).cloned()
    }

    // Update
    //
    pub fn update(&mut self, mut new_record: R) {
        let id = new_record.id();

        let old_ref = self
            .records
            .iter_mut()
            .find(|r| r.id() == id)
            .unwrap_or_else(|| {
                let typ = base_type_name::<R>();
                panic!(".update() failed, because {typ} (id={id}) does not exist");
            });

        let new_ref = &mut new_record;
        std::mem::swap(old_ref, new_ref);
    }

    // Delete
    //
    pub fn delete(&mut self, id: <R as Identifiable>::Id) {
        let index = self
            .records
            .iter()
            .position(|r| r.id() == id)
            .unwrap_or_else(|| {
                let typ = base_type_name::<R>();
                panic!(".delete() failed: could not find {typ} (id={id})");
            });
        self.records.remove(index);
    }
}

fn base_type_name<T>() -> &'static str {
    let full_name = std::any::type_name::<T>();
    full_name.split_terminator("::").last().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::impl_identifiable;

    #[derive(Debug, Clone, PartialEq)]
    struct User {
        id: i32,
        name: String,
    }
    impl_identifiable!(User, i32);

    fn bob() -> User {
        User {
            id: 1,
            name: "Bob".to_owned(),
        }
    }

    fn alice() -> User {
        User {
            id: 2,
            name: "Alice".to_owned(),
        }
    }

    fn tom() -> User {
        User {
            id: 3,
            name: "Tom".to_owned(),
        }
    }

    #[test]
    fn test_insert() {
        let mut state = vec![];
        let mut rel = Relation::new(&mut state);
        rel.insert(bob());
        rel.insert(alice());
        assert_eq!(state, vec![bob(), alice()]);
    }

    #[test]
    #[should_panic(
        expected = "Cannot insert User (id=2), because a record with this ID already exists"
    )]
    fn test_insert_panics_when_id_is_duplicated() {
        let mut state = vec![bob(), alice()];
        let mut rel = Relation::new(&mut state);
        rel.insert(alice());
    }

    #[test]
    fn test_insert_many() {
        let mut state = vec![];
        let mut rel = Relation::new(&mut state);
        rel.insert_many(vec![bob(), alice(), tom()]);
        assert_eq!(state, vec![bob(), alice(), tom()]);
    }

    #[test]
    fn test_get_all() {
        let mut state = vec![alice(), tom()];
        let rel = Relation::new(&mut state);
        assert_eq!(rel.get_all(), vec![alice(), tom()]);
    }

    #[test]
    fn test_get_by_id() {
        let mut state = vec![alice(), tom()];
        let rel = Relation::new(&mut state);
        assert_eq!(rel.get_by_id(1), None);
        assert_eq!(rel.get_by_id(2), Some(alice()));
        assert_eq!(rel.get_by_id(3), Some(tom()));
    }

    #[test]
    fn test_update() {
        let mut state = vec![alice(), bob(), tom()];
        let mut rel = Relation::new(&mut state);

        let robert = User {
            name: "Robert".to_owned(),
            ..bob()
        };
        rel.update(robert.clone());
        assert_eq!(state, vec![alice(), robert, tom(),]);
    }

    #[test]
    fn test_delete() {
        let mut state = vec![alice(), bob(), tom()];
        let mut rel = Relation::new(&mut state);

        rel.delete(bob().id);
        assert_eq!(state, vec![alice(), tom(),]);
    }
}
