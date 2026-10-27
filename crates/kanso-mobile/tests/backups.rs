//! The library's backups through the facade: the daily one, the one made on
//! request and kept out of the rotation, and what may be deleted.

mod common;

use std::path::Path;

use common::*;
use kanso_mobile::{BACKUPS_SUMMARY, KansoError, diagnostics::backup_excluded};
use proptest::prelude::*;

// no-prop-test: the daily backup is made once; the second call is the same day.
#[test]
fn the_daily_backup_is_made_once_a_day() {
    let phone = lone_phone();
    let library = &phone.library;
    note(library, "Plan", "text");
    let first = library
        .backup_if_due()
        .unwrap()
        .expect("the first one is due");
    assert!(!first.manual && first.bytes > 0);
    assert_eq!(library.backup_if_due().unwrap(), None);
    assert_eq!(library.backups().unwrap(), vec![first]);
}

// no-prop-test: a backup on request among daily ones.
#[test]
fn a_backup_on_request_stays_out_of_the_rotation() {
    let phone = lone_phone();
    let library = &phone.library;
    let manual = library.back_up_now().unwrap();
    assert!(manual.manual);
    // It doesn't stand in for the day's backup.
    let daily = library
        .backup_if_due()
        .unwrap()
        .expect("the daily one is still due");
    assert!(!daily.manual);
    std::thread::sleep(std::time::Duration::from_millis(3));
    let second = library.back_up_now().unwrap();
    let listed = library.backups().unwrap();
    assert_eq!(listed.len(), 3);
    assert!(
        listed
            .windows(2)
            .all(|w| w[0].made_at_ms >= w[1].made_at_ms)
    );
    assert_eq!(listed.iter().filter(|b| b.manual).count(), 2);
    assert_eq!(listed[0], second);
}

// no-prop-test: the refusals, one each.
#[test]
fn only_a_backup_made_on_request_can_be_deleted() {
    let phone = lone_phone();
    let library = &phone.library;
    let daily = library.backup_if_due().unwrap().unwrap();
    let manual = library.back_up_now().unwrap();
    let outside = phone.dir.path().join("kanso-manual-1.db");
    std::fs::write(&outside, b"not a backup of this folder").unwrap();
    let elsewhere = phone
        .dir
        .path()
        .join("backups")
        .join("..")
        .join("kanso-manual-1.db");

    for refused in [
        daily.path.clone(),
        outside.to_string_lossy().into_owned(),
        elsewhere.to_string_lossy().into_owned(),
        phone.dir.path().to_string_lossy().into_owned(),
        String::new(),
    ] {
        assert!(
            matches!(
                library.delete_backup(&refused),
                Err(KansoError::Invalid { .. })
            ),
            "{refused}"
        );
    }
    assert!(Path::new(&daily.path).exists() && outside.exists());
    library.delete_backup(&manual.path).unwrap();
    assert!(!Path::new(&manual.path).exists());
    assert_eq!(
        library.delete_backup(&manual.path),
        Err(KansoError::NotFound)
    );
    assert_eq!(library.backups().unwrap(), vec![daily]);
}

// no-prop-test: where the folder is and what the screen says.
#[test]
fn backups_sit_inside_the_library_folder_and_the_phone_words_are_the_phones() {
    let phone = lone_phone();
    let library = &phone.library;
    let made = library.back_up_now().unwrap();
    let folder = library.path().parent().unwrap().to_path_buf();
    assert!(Path::new(&made.path).starts_with(&folder));
    assert_eq!(
        Path::new(&made.path).parent().unwrap(),
        folder.join("backups")
    );
    // The folder is the one kept out of the device's backups, so what is
    // inside it is too.
    if cfg!(target_vendor = "apple") {
        assert_eq!(backup_excluded(&folder), Some(true));
    }
    assert!(BACKUPS_SUMMARY.starts_with("Kanso backs up its database on this device once a day"));
    assert!(BACKUPS_SUMMARY.ends_with("kept until you delete it here."));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    /// However many backups are made on request, each is listed once with
    /// the size of its file, and the next daily one is still made.
    #[test]
    fn prop_every_backup_on_request_is_listed_and_kept(count in 1usize..4) {
        let phone = lone_phone();
        let library = &phone.library;
        note(library, "Plan", "text");
        let mut made = Vec::new();
        for _ in 0..count {
            made.push(library.back_up_now().unwrap());
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        prop_assert!(library.backup_if_due().unwrap().is_some());
        let listed = library.backups().unwrap();
        prop_assert_eq!(listed.len(), count + 1);
        for backup in &made {
            let found = listed.iter().find(|b| b.path == backup.path).unwrap();
            prop_assert_eq!(found.bytes, std::fs::metadata(&backup.path).unwrap().len());
            prop_assert!(found.manual);
        }
    }
}
