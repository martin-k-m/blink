use std::fs;

use tempfile::TempDir;

use crate::Cache;

#[test]
fn first_scan_reports_everything_as_added() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.txt"), "hello").unwrap();
    fs::write(dir.path().join("b.txt"), "world").unwrap();

    let empty = Cache::default();
    let scanned = Cache::scan(dir.path());
    let diff = scanned.diff(&empty);

    assert_eq!(diff.total, 2);
    assert_eq!(diff.added, 2);
    assert_eq!(diff.unchanged, 0);
}

#[test]
fn unchanged_files_are_detected_across_scans() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.txt"), "hello").unwrap();
    fs::write(dir.path().join("b.txt"), "world").unwrap();

    let first = Cache::scan(dir.path());
    let second = Cache::scan(dir.path());
    let diff = second.diff(&first);

    assert_eq!(diff.unchanged, 2);
    assert_eq!(diff.changed, 0);
    assert_eq!(diff.added, 0);
}

#[test]
fn modified_file_is_detected_as_changed() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.txt"), "hello").unwrap();

    let first = Cache::scan(dir.path());
    fs::write(dir.path().join("a.txt"), "goodbye").unwrap();
    let second = Cache::scan(dir.path());
    let diff = second.diff(&first);

    assert_eq!(diff.changed, 1);
    assert_eq!(diff.unchanged, 0);
}

#[test]
fn removed_file_is_detected() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.txt"), "hello").unwrap();
    fs::write(dir.path().join("b.txt"), "world").unwrap();

    let first = Cache::scan(dir.path());
    fs::remove_file(dir.path().join("b.txt")).unwrap();
    let second = Cache::scan(dir.path());
    let diff = second.diff(&first);

    assert_eq!(diff.removed, 1);
    assert_eq!(diff.total, 1);
}

#[test]
fn cache_round_trips_through_disk() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.txt"), "hello").unwrap();

    let scanned = Cache::scan(dir.path());
    scanned.save(dir.path()).unwrap();

    let loaded = Cache::load(dir.path())
        .unwrap()
        .expect("cache should exist");
    let diff = scanned.diff(&loaded);

    assert_eq!(diff.unchanged, 1);
}

#[test]
fn load_returns_none_when_no_cache_exists() {
    let dir = TempDir::new().unwrap();
    assert!(Cache::load(dir.path()).unwrap().is_none());
}
