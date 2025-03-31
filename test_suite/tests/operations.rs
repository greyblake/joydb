use test_suite::helpers::with_open_db;
use toydb::ToydbError;
use uuid::Uuid;

use test_suite::database::{Database, Post, User};
use test_suite::helpers::gen_db_file_path;

#[test]
fn should_insert_and_find() {
    with_open_db(|db| {
        let alice = User {
            id: Uuid::new_v4(),
            name: "Alice".to_string(),
        };
        let alice_id = alice.id;

        let bob = User {
            id: Uuid::new_v4(),
            name: "Bob".to_string(),
        };

        db.insert(alice).unwrap();
        db.insert(bob).unwrap();

        assert_eq!(db.count::<User>().unwrap(), 2);
        assert_eq!(db.count::<Post>().unwrap(), 0);

        let alice = db.find::<User>(&alice_id).unwrap();
        assert_eq!(alice.name, "Alice");
    });
}

#[test]
fn should_return_error_on_attempt_to_insert_record_with_duplicated_id() {
    with_open_db(|db| {
        let alice = User {
            id: Uuid::new_v4(),
            name: "Alice".to_string(),
        };
        let alice_id = alice.id;
        db.insert(alice).unwrap();

        let another_alice = User {
            id: alice_id,
            name: "Another Alice".to_string(),
        };

        // Check the error
        let err = db.insert(another_alice).unwrap_err();
        assert!(matches!(err, ToydbError::DuplicatedId { .. }));
        assert_eq!(
            err.to_string(),
            format!("User with id = {alice_id} already exists")
        );

        // Make sure we did not add any new records
        assert_eq!(db.count::<User>().unwrap(), 1);

        let same_alice = db.find::<User>(&alice_id).unwrap();
        assert_eq!(same_alice.name, "Alice");
    });
}
