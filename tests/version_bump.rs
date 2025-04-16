use assert_cmd::Command;
use predicates::str::contains;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Once,
};

static INIT: Once = Once::new();
static CLEANUP: Once = Once::new();


fn init_tmp_root() {
    INIT.call_once(|| {
        let _ = fs::create_dir_all("./tmp-test");

        // Register a cleanup guard that triggers when tests end
        CLEANUP.call_once(|| {
            std::panic::set_hook(Box::new(|info| {
                // Also try cleanup on panic to avoid leftover folders
                let _ = fs::remove_dir_all("./tmp-test");
                eprintln!("Test panicked: {:?}", info);
            }));

            // Use Drop to clean at the very end of the test process
            std::thread::spawn(|| {
                struct TmpTestCleaner;
                impl Drop for TmpTestCleaner {
                    fn drop(&mut self) {
                        let _ = fs::remove_dir_all("./tmp-test");
                    }
                }

                let _guard = TmpTestCleaner;

                // Keep thread alive until test binary exits
                std::thread::sleep(std::time::Duration::from_secs(60));
            });
        });
    });
}

fn make_test_dir(name: &str) -> PathBuf {
    init_tmp_root();
    let path = PathBuf::from(format!("./tmp-test/{}", name));
    if path.exists() {
        let _ = fs::remove_dir_all(&path);
    }
    let _ = fs::create_dir_all(&path);
    path
}

fn write_file(path: &Path, filename: &str, content: &str) {
    fs::write(path.join(filename), content).expect("Failed to write test file");
}


#[test]
fn test_bump_with_composer_json_priority() {
    let path = make_test_dir("composer-priority");

    write_file(&path, "composer.json", r#"{ "version": "0.1.0" }"#);
    write_file(&path, "package.json", r#"{ "version": "9.9.9" }"#);
    write_file(&path, "VERSION", "2.2.2");

    let mut cmd = Command::cargo_bin("semver").unwrap();
    cmd.current_dir(&path)
        .arg("--bump")
        .arg("patch")
        .assert()
        .success()
        .stdout(contains("0.1.0 → 0.1.1"));

    let updated = fs::read_to_string(path.join("composer.json")).unwrap();
    assert!(updated.contains("\"version\": \"0.1.1\""));
}

#[test]
fn test_bump_with_package_json_priority() {
    let path = make_test_dir("package-priority");

    write_file(&path, "package.json", r#"{ "version": "1.0.5" }"#);
    write_file(&path, "VERSION", "3.3.3");

    let mut cmd = Command::cargo_bin("semver").unwrap();
    cmd.current_dir(&path)
        .arg("--bump")
        .arg("patch")
        .assert()
        .success()
        .stdout(contains("1.0.5 → 1.0.6"));

    let updated = fs::read_to_string(path.join("package.json")).unwrap();
    assert!(updated.contains("\"version\": \"1.0.6\""));
}

#[test]
fn test_bump_with_version_file_only() {
    let path = make_test_dir("version-priority");

    write_file(&path, "VERSION", "0.0.9");

    let mut cmd = Command::cargo_bin("semver").unwrap();
    cmd.current_dir(&path)
        .arg("--bump")
        .arg("patch")
        .assert()
        .success()
        .stdout(contains("0.0.9 → 0.0.10"));

    let updated = fs::read_to_string(path.join("VERSION")).unwrap();
    assert_eq!(updated.trim(), "0.0.10");
}

#[test]
fn test_does_not_update_package_json_without_version_field() {
    let path = make_test_dir("no-package-version");

    write_file(&path, "package.json", r#"{ "name": "no-version" }"#);
    write_file(&path, "VERSION", "0.2.1");

    let mut cmd = Command::cargo_bin("semver").unwrap();
    cmd.current_dir(&path)
        .arg("--bump")
        .arg("patch")
        .assert()
        .success()
        .stdout(contains("0.2.1 → 0.2.2"));

    let package_json = fs::read_to_string(path.join("package.json")).unwrap();
    assert!(!package_json.contains("0.2.2")); // no version added
}

#[test]
fn test_no_version_file_exists() {
    let path = make_test_dir("version-files");

    let mut cmd = Command::cargo_bin("semver").unwrap();
    cmd.current_dir(path)
        .assert()
        .failure()
        .stderr(contains("No version found"));
}
