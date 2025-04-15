use joydb::JoydbError;
use test_suite::helpers::with_open_db;
use uuid::Uuid;

use test_suite::database::{Post, User};

#[test]
fn should_insert_and_find() {
    with_open_db(|db| {
        let alice = User {
            id: Uuid::new_v4(),
            name: "Alice".to_string(),
            age: 30,
        };
        let alice_id = alice.id;

        let bob = User {
            id: Uuid::new_v4(),
            name: "Bob".to_string(),
            age: 25,
        };

        db.insert(&alice).unwrap();
        db.insert(&bob).unwrap();

        assert_eq!(db.count::<User>().unwrap(), 2);
        assert_eq!(db.count::<Post>().unwrap(), 0);

        let alice = db.get::<User>(&alice_id).unwrap().unwrap();
        assert_eq!(alice.name, "Alice");
    });
}

#[test]
fn should_return_error_on_attempt_to_insert_record_with_duplicated_id() {
    with_open_db(|db| {
        let alice = User {
            id: Uuid::new_v4(),
            name: "Alice".to_string(),
            age: 30,
        };
        let alice_id = alice.id;
        db.insert(&alice).unwrap();

        let another_alice = User {
            id: alice_id,
            name: "Another Alice".to_string(),
            age: 25,
        };

        // Check the error
        let err = db.insert(&another_alice).unwrap_err();
        assert!(matches!(err, JoydbError::DuplicatedId { .. }));
        assert_eq!(
            err.to_string(),
            format!("User with id = {alice_id} already exists")
        );

        // Make sure we did not add any new records
        assert_eq!(db.count::<User>().unwrap(), 1);

        let same_alice = db.get::<User>(&alice_id).unwrap().unwrap();
        assert_eq!(same_alice.name, "Alice");
    });
}

#[test]
fn should_update() {
    with_open_db(|db| {
        let alice = User {
            id: Uuid::new_v4(),
            name: "Alice".to_string(),
            age: 30,
        };
        let alice_id = alice.id;
        db.insert(&alice).unwrap();

        let alice = db.get::<User>(&alice_id).unwrap().unwrap();
        assert_eq!(alice.name, "Alice");

        let alice = User {
            id: alice_id,
            name: "Alice Updated".to_string(),
            age: 31,
        };
        db.update(alice).unwrap();

        let alice = db.get::<User>(&alice_id).unwrap().unwrap();
        assert_eq!(alice.name, "Alice Updated");
        assert_eq!(alice.age, 31);
    });
}

#[test]
fn should_return_error_on_update_if_record_does_not_exist() {
    with_open_db(|db| {
        let alice = User {
            id: Uuid::new_v4(),
            name: "Alice".to_string(),
            age: 30,
        };
        let alice_id = alice.id;

        let err = db.update(alice).unwrap_err();
        assert!(matches!(err, JoydbError::NotFound { .. }));
        assert_eq!(
            err.to_string(),
            format!("User with id = {alice_id} not found")
        );
    });
}

#[test]
fn should_get_all_records_that_match_given_predicate() {
    with_open_db(|db| {
        let alice = User {
            id: Uuid::new_v4(),
            name: "Alice".to_string(),
            age: 30,
        };
        db.insert(&alice).unwrap();

        let bob = User {
            id: Uuid::new_v4(),
            name: "Bob".to_string(),
            age: 25,
        };
        db.insert(&bob).unwrap();

        let charlie = User {
            id: Uuid::new_v4(),
            name: "Charlie".to_string(),
            age: 15,
        };
        db.insert(&charlie).unwrap();

        let kids: Vec<User> = db.get_all_by(|u: &User| u.age < 18).unwrap();
        assert_eq!(kids.len(), 1);
        assert_eq!(kids[0].name, "Charlie");

        let adults: Vec<User> = db.get_all_by(|u: &User| u.age >= 18).unwrap();
        let adult_names: Vec<String> = adults.iter().map(|u| u.name.clone()).collect();
        assert_eq!(adult_names.len(), 2);
        assert!(adult_names.contains(&"Alice".to_string()));
        assert!(adult_names.contains(&"Bob".to_string()));
    });
}

#[test]
fn should_delete_all_records_that_match_predicate() {
    with_open_db(|db| {
        let alice = User {
            id: Uuid::new_v4(),
            name: "Alice".to_string(),
            age: 30,
        };
        db.insert(&alice).unwrap();

        let bob = User {
            id: Uuid::new_v4(),
            name: "Bob".to_string(),
            age: 25,
        };
        db.insert(&bob).unwrap();


        let deleted_users = db.delete_all_by(|u: &User| u.age > 27).unwrap();
        assert_eq!(deleted_users.len(), 1);
        assert_eq!(deleted_users[0].name, "Alice");

        let remaining_users: Vec<User> = db.get_all().unwrap();
        assert_eq!(remaining_users.len(), 1);
        assert_eq!(remaining_users[0].name, "Bob");
    });
}
