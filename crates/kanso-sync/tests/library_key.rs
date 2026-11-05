//! Opening a library with the key the secret store keeps (issue #243 step 9).

use kanso_store::{LibraryFile, Store, StoreError};
use kanso_sync::keychain::library_key_item;
use kanso_sync::library_key::{is_key_id, key_id_path, load_key};
use kanso_sync::{Encryption, MemorySecrets, OpenLibraryError, SecretStore, open_library};
use proptest::prelude::*;

fn db(dir: &tempfile::TempDir) -> std::path::PathBuf {
    dir.path().join("kanso.db")
}

fn key_id(dir: &tempfile::TempDir) -> String {
    std::fs::read_to_string(key_id_path(&db(dir))).unwrap()
}

#[test]
fn a_new_library_is_encrypted_and_reopens_with_the_saved_key() {
    let dir = tempfile::tempdir().unwrap();
    let secrets = MemorySecrets::new();
    let opened = open_library(&db(&dir), &secrets).unwrap();
    assert_eq!(opened.encryption, Encryption::Encrypted);
    opened.store.set_meta("hello", "world").unwrap();
    drop(opened);

    assert_eq!(LibraryFile::of(&db(&dir)), LibraryFile::Encrypted);
    let id = key_id(&dir);
    assert!(is_key_id(&id));
    assert_eq!(secrets.items(), vec![library_key_item(&id)]);
    assert!(load_key(&secrets, &id).unwrap().is_some());

    let again = open_library(&db(&dir), &secrets).unwrap();
    assert_eq!(again.encryption, Encryption::Encrypted);
    assert_eq!(again.store.meta("hello").unwrap().as_deref(), Some("world"));
}

#[test]
fn an_existing_plain_library_opens_as_it_always_did() {
    let dir = tempfile::tempdir().unwrap();
    Store::open(db(&dir)).unwrap().set_meta("a", "b").unwrap();
    let secrets = MemorySecrets::new();
    let opened = open_library(&db(&dir), &secrets).unwrap();
    assert_eq!(opened.encryption, Encryption::Plain);
    assert_eq!(opened.store.meta("a").unwrap().as_deref(), Some("b"));
    // Nothing was made: no key, no item, still plain.
    drop(opened);
    assert!(secrets.items().is_empty());
    assert!(!key_id_path(&db(&dir)).exists());
    assert_eq!(LibraryFile::of(&db(&dir)), LibraryFile::Plain);
}

#[test]
fn an_encrypted_library_without_its_key_does_not_open() {
    let dir = tempfile::tempdir().unwrap();
    let secrets = MemorySecrets::new();
    drop(open_library(&db(&dir), &secrets).unwrap());
    let id = key_id(&dir);

    // Another computer: the file, but not the key.
    let elsewhere = MemorySecrets::new();
    let err = open_library(&db(&dir), &elsewhere).err().unwrap();
    assert!(matches!(err, OpenLibraryError::Store(StoreError::WrongKey)));
    assert_eq!(
        err.to_string(),
        "Kanso can't open this library with this computer's key."
    );

    // The key item replaced by another library's key.
    let other = tempfile::tempdir().unwrap();
    drop(open_library(&db(&other), &elsewhere).unwrap());
    let stolen = load_key(&elsewhere, &key_id(&other)).unwrap().unwrap();
    kanso_sync::library_key::save_key(&elsewhere, &id, &stolen).unwrap();
    assert!(matches!(
        open_library(&db(&dir), &elsewhere).err().unwrap(),
        OpenLibraryError::Store(StoreError::WrongKey)
    ));

    // No key ID file.
    std::fs::remove_file(key_id_path(&db(&dir))).unwrap();
    assert!(matches!(
        open_library(&db(&dir), &secrets).err().unwrap(),
        OpenLibraryError::Store(StoreError::WrongKey)
    ));
}

#[test]
fn a_locked_keychain_is_not_taken_for_a_missing_key() {
    let dir = tempfile::tempdir().unwrap();
    let secrets = MemorySecrets::new();
    drop(open_library(&db(&dir), &secrets).unwrap());
    secrets.fail_reads(true);
    let err = open_library(&db(&dir), &secrets).err().unwrap();
    assert!(matches!(err, OpenLibraryError::Keychain(_)), "{err}");
    secrets.fail_reads(false);
    open_library(&db(&dir), &secrets).unwrap();
}

#[test]
fn a_secret_store_that_refuses_gives_a_plain_new_library_and_no_orphans() {
    let dir = tempfile::tempdir().unwrap();
    let secrets = MemorySecrets::new();
    secrets.fail_writes_to(Some("library-key:"));
    let opened = open_library(&db(&dir), &secrets).unwrap();
    assert!(matches!(opened.encryption, Encryption::PlainNoKeychain(_)));
    drop(opened);
    assert_eq!(LibraryFile::of(&db(&dir)), LibraryFile::Plain);
    assert!(secrets.items().is_empty());
    assert!(!key_id_path(&db(&dir)).exists());
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 16, ..ProptestConfig::default() })]

    /// Invariants: each new library gets its own key and key ID; the keys of
    /// other libraries never open it.
    #[test]
    fn prop_one_libraries_key_does_not_open_another(n in 2usize..4) {
        let secrets = MemorySecrets::new();
        let dirs: Vec<_> = (0..n).map(|_| tempfile::tempdir().unwrap()).collect();
        for dir in &dirs {
            drop(open_library(&db(dir), &secrets).unwrap());
        }
        let ids: Vec<String> = dirs.iter().map(key_id).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        prop_assert_eq!(sorted.len(), n);
        for (i, dir) in dirs.iter().enumerate() {
            let own = load_key(&secrets, &ids[i]).unwrap().unwrap();
            drop(Store::open_with_key(db(dir), Some(&own)).unwrap());
            for (j, id) in ids.iter().enumerate() {
                if i != j {
                    let foreign = load_key(&secrets, id).unwrap().unwrap();
                    prop_assert!(matches!(
                        Store::open_with_key(db(dir), Some(&foreign)),
                        Err(StoreError::WrongKey)
                    ));
                }
            }
        }
        // Reading the items back never needs more than the secret store.
        let _: &dyn SecretStore = &secrets;
    }
}
