//! The whole update on this machine, with nothing real touched: a fake
//! Kanso in a temporary folder, a feed signed with a test key and served
//! from a local HTTP server on an ephemeral port (through a redirect, as
//! GitHub serves it), then check, download, verify, stage and swap.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use ed25519_dalek::SigningKey;
use kanso_update::install::{self, LocateOptions};
use kanso_update::{
    Asset, Check, CheckError, Feed, FeedError, HttpTransport, Platform, SCHEMA, StageError,
    UrlPolicy, sign_feed, stage,
};
use sha2::{Digest, Sha256};

const PLATFORM: Platform = match Platform::current() {
    Some(p) => p,
    None => Platform::LinuxX64,
};

/// Serves `routes` (path → answer) until the test ends.
fn serve(routes: Vec<(String, Answer)>) -> String {
    let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").unwrap());
    let addr = server.server_addr().to_ip().unwrap();
    std::thread::spawn(move || {
        for request in server.incoming_requests() {
            let path = request.url().to_owned();
            let response = match routes.iter().find(|(p, _)| *p == path).map(|(_, a)| a) {
                Some(Answer::Bytes(bytes)) => tiny_http::Response::from_data(bytes.clone()).boxed(),
                Some(Answer::Redirect(to)) => tiny_http::Response::empty(302)
                    .with_header(tiny_http::Header::from_bytes("Location", to.as_bytes()).unwrap())
                    .boxed(),
                Some(Answer::Status(code)) => tiny_http::Response::empty(*code).boxed(),
                None => tiny_http::Response::empty(404).boxed(),
            };
            request.respond(response).ok();
        }
    });
    format!("http://{addr}")
}

#[derive(Clone)]
enum Answer {
    Bytes(Vec<u8>),
    Redirect(String),
    Status(u16),
}

/// A minimal Kanso install of `version` in `dir`: a signed `.app` on
/// macOS, an executable elsewhere. Returns the executable.
fn fake_kanso(dir: &Path, version: &str) -> PathBuf {
    if PLATFORM.is_bundle() {
        let app = dir.join("Kanso.app");
        let macos = app.join("Contents/MacOS");
        std::fs::create_dir_all(&macos).unwrap();
        std::fs::copy("/usr/bin/true", macos.join("kanso")).unwrap();
        std::fs::write(
            app.join("Contents/Info.plist"),
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>app.kanso.test</string>
<key>CFBundleExecutable</key><string>kanso</string>
<key>CFBundleShortVersionString</key><string>{version}</string>
</dict></plist>
"#
            ),
        )
        .unwrap();
        codesign(&app);
        macos.join("kanso")
    } else {
        let exe = dir.join(if PLATFORM == Platform::WindowsX64 {
            "kanso.exe"
        } else {
            "kanso"
        });
        std::fs::write(&exe, format!("kanso {version}")).unwrap();
        exe
    }
}

#[cfg(target_os = "macos")]
fn codesign(app: &Path) {
    let status = std::process::Command::new("/usr/bin/codesign")
        .args(["--force", "--sign", "-"])
        .arg(app)
        .output()
        .unwrap();
    assert!(status.status.success());
}

#[cfg(not(target_os = "macos"))]
fn codesign(_: &Path) {}

/// The release archive of a fake Kanso `version`, as the workflow packs it.
fn release_archive(version: &str) -> Vec<u8> {
    let build = tempfile::tempdir().unwrap();
    let exe = fake_kanso(build.path(), version);
    match PLATFORM.archive() {
        kanso_update::ArchiveKind::TarGz => {
            let gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
            let mut tar = tar::Builder::new(gz);
            if PLATFORM.is_bundle() {
                tar.append_dir_all("Kanso.app", build.path().join("Kanso.app"))
                    .unwrap();
            } else {
                let stem = format!("Kanso-{version}-{}", PLATFORM.key());
                tar.append_path_with_name(&exe, format!("{stem}/kanso"))
                    .unwrap();
            }
            tar.into_inner().unwrap().finish().unwrap()
        }
        kanso_update::ArchiveKind::Zip => {
            let mut out = std::io::Cursor::new(Vec::new());
            let mut zip = zip::ZipWriter::new(&mut out);
            zip.start_file("kanso.exe", zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(&std::fs::read(&exe).unwrap()).unwrap();
            zip.finish().unwrap();
            out.into_inner()
        }
    }
}

fn installed_version(target: &Path) -> String {
    if PLATFORM.is_bundle() {
        let plist = std::fs::read_to_string(target.join("Contents/Info.plist")).unwrap();
        let at = plist
            .find("CFBundleShortVersionString</key><string>")
            .unwrap()
            + 40;
        plist[at..plist[at..].find('<').unwrap() + at].to_owned()
    } else {
        std::fs::read_to_string(target)
            .unwrap()
            .trim_start_matches("kanso ")
            .to_owned()
    }
}

struct Release {
    base: String,
    key: SigningKey,
}

/// Serves release `version` with `archive`; the feed is at
/// `<base>/releases/latest/download/latest.json`, a redirect to the tag's.
fn publish(
    version: &str,
    archive: Vec<u8>,
    tamper: impl FnOnce(&mut Feed, &mut Vec<u8>),
) -> Release {
    let key = SigningKey::from_bytes(&[7; 32]);
    // The routes need the address, which needs the server: two passes.
    let placeholder = "http://127.0.0.1:1";
    let mut feed = Feed {
        schema: SCHEMA,
        version: version.into(),
        pub_date: "2026-10-20T10:00:00Z".into(),
        release_url: format!("{placeholder}/releases/tag/v{version}"),
        notes: "### New\n\n- Updates install themselves".into(),
        platforms: [(
            PLATFORM.key().to_owned(),
            Asset {
                url: String::new(),
                sha256: Sha256::digest(&archive)
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect(),
                size: archive.len() as u64,
            },
        )]
        .into(),
        whats_new: None,
        earlier: Vec::new(),
    };
    let mut archive = archive;
    tamper(&mut feed, &mut archive);
    let routes = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let base = {
        let routes = routes.clone();
        let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").unwrap());
        let addr = server.server_addr().to_ip().unwrap();
        std::thread::spawn(move || {
            for request in server.incoming_requests() {
                let path = request.url().to_owned();
                let answer = routes
                    .lock()
                    .unwrap()
                    .iter()
                    .find(|(p, _): &&(String, Answer)| *p == path)
                    .map(|(_, a)| a.clone());
                let response = match answer {
                    Some(Answer::Bytes(bytes)) => tiny_http::Response::from_data(bytes).boxed(),
                    Some(Answer::Redirect(to)) => tiny_http::Response::empty(302)
                        .with_header(
                            tiny_http::Header::from_bytes("Location", to.as_bytes()).unwrap(),
                        )
                        .boxed(),
                    Some(Answer::Status(code)) => tiny_http::Response::empty(code).boxed(),
                    None => tiny_http::Response::empty(404).boxed(),
                };
                request.respond(response).ok();
            }
        });
        format!("http://{addr}")
    };
    feed.release_url = feed.release_url.replace(placeholder, &base);
    let asset_path = format!(
        "/releases/download/v{version}/{}",
        PLATFORM.asset_name(version)
    );
    if let Some(asset) = feed.platforms.get_mut(PLATFORM.key())
        && asset.url.is_empty()
    {
        asset.url = format!("{base}{asset_path}");
    }
    let bytes = serde_json::to_vec_pretty(&feed).unwrap();
    let sig = sign_feed(&bytes, &key).into_bytes();
    let tag = format!("/releases/download/v{version}");
    *routes.lock().unwrap() = vec![
        (
            "/releases/latest/download/latest.json".into(),
            Answer::Redirect(format!("{tag}/latest.json")),
        ),
        (
            "/releases/latest/download/latest.json.sig".into(),
            Answer::Redirect(format!("{tag}/latest.json.sig")),
        ),
        (format!("{tag}/latest.json"), Answer::Bytes(bytes)),
        (format!("{tag}/latest.json.sig"), Answer::Bytes(sig)),
        (asset_path, Answer::Bytes(archive)),
    ];
    Release { base, key }
}

const LOCAL: UrlPolicy = UrlPolicy {
    repository: kanso_update::REPOSITORY,
    allow_loopback: true,
};

fn check(release: &Release, current: &str) -> Result<Check, CheckError> {
    kanso_update::check(
        &HttpTransport::new("kanso-test"),
        &LOCAL,
        &format!("{}/releases/latest/download/latest.json", release.base),
        &[release.key.verifying_key()],
        &semver::Version::parse(current).unwrap(),
        PLATFORM,
    )
}

fn verify(payload: &Path, running: &Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    return install::verify_bundle(payload, Some(running));
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (payload, running);
        Ok(())
    }
}

#[test]
fn check_download_verify_stage_and_swap() {
    let dir = tempfile::tempdir().unwrap();
    let exe = fake_kanso(dir.path(), "0.1.0");
    let release = publish("0.2.0", release_archive("0.2.0"), |_, _| {});

    let Check::Available(update) = check(&release, "0.1.0").unwrap() else {
        panic!("0.2.0 is newer than 0.1.0");
    };
    assert_eq!(update.version, "0.2.0");

    let install = install::locate(&exe, Some(PLATFORM), LocateOptions::default()).unwrap();
    let mut seen = 0;
    let pending = stage::prepare(
        &HttpTransport::new("kanso-test"),
        &LOCAL,
        &update,
        &install,
        &mut |done, total| {
            assert!(done <= total);
            seen = done;
        },
        &AtomicBool::new(false),
        &|payload| verify(payload, &install.target),
    )
    .unwrap();
    assert_eq!(seen, update.asset.size);
    assert!(pending.payload.starts_with(&install.stage_dir));
    assert!(pending.seal.is_set());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = std::fs::metadata(&install.stage_dir)
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o700, "the staging folder is private");
    }
    // Staged beside the install, untouched until the swap.
    assert_eq!(installed_version(&install.target), "0.1.0");

    install::apply(&pending, &install).unwrap();
    assert_eq!(installed_version(&install.target), "0.2.0");
    assert!(!install.stage_dir.exists());
    #[cfg(target_os = "macos")]
    install::verify_bundle(&install.target, None).unwrap();

    // Now on 0.2.0, the same feed has nothing newer.
    assert_eq!(
        check(&release, "0.2.0").unwrap(),
        Check::UpToDate {
            latest: "0.2.0".into()
        }
    );
}

#[test]
fn a_tampered_archive_is_never_staged() {
    let dir = tempfile::tempdir().unwrap();
    let exe = fake_kanso(dir.path(), "0.1.0");
    // The feed signs the real archive's checksum; the server sends another.
    let release = publish("0.2.0", release_archive("0.2.0"), |_, archive| {
        let last = archive.len() - 1;
        archive[last] ^= 1;
    });
    let Check::Available(update) = check(&release, "0.1.0").unwrap() else {
        panic!()
    };
    let install = install::locate(&exe, Some(PLATFORM), LocateOptions::default()).unwrap();
    let err = stage::prepare(
        &HttpTransport::new("kanso-test"),
        &LOCAL,
        &update,
        &install,
        &mut |_, _| {},
        &AtomicBool::new(false),
        &|_| Ok(()),
    )
    .unwrap_err();
    assert!(matches!(err, StageError::ChecksumMismatch), "{err:?}");
    assert_eq!(
        std::fs::read_dir(&install.stage_dir).unwrap().count(),
        0,
        "nothing is left staged"
    );
    assert_eq!(installed_version(&install.target), "0.1.0");
}

#[test]
fn an_update_changed_after_its_checks_is_never_swapped_in() {
    let dir = tempfile::tempdir().unwrap();
    let exe = fake_kanso(dir.path(), "0.1.0");
    let release = publish("0.2.0", release_archive("0.2.0"), |_, _| {});
    let Check::Available(update) = check(&release, "0.1.0").unwrap() else {
        panic!()
    };
    let install = install::locate(&exe, Some(PLATFORM), LocateOptions::default()).unwrap();
    let pending = stage::prepare(
        &HttpTransport::new("kanso-test"),
        &LOCAL,
        &update,
        &install,
        &mut |_, _| {},
        &AtomicBool::new(false),
        &|payload| verify(payload, &install.target),
    )
    .unwrap();
    // Something with write access to the staging folder swaps the checked
    // executable for its own while Kanso runs.
    let executable = if PLATFORM.is_bundle() {
        pending.payload.join("Contents/MacOS/kanso")
    } else {
        pending.payload.clone()
    };
    std::fs::write(&executable, b"#!/bin/sh\necho pwned\n").unwrap();
    assert!(matches!(
        install::apply(&pending, &install),
        Err(install::ApplyError::Tampered)
    ));
    assert_eq!(installed_version(&install.target), "0.1.0");
    // And on the next start, whatever was staged is cleared, not offered.
    install::clean_up(&install);
    assert!(!install.stage_dir.exists());
}

#[test]
fn a_feed_signed_by_another_key_is_refused() {
    let release = publish("0.2.0", release_archive("0.2.0"), |_, _| {});
    let other = SigningKey::from_bytes(&[9; 32]);
    let err = kanso_update::check(
        &HttpTransport::new("kanso-test"),
        &LOCAL,
        &format!("{}/releases/latest/download/latest.json", release.base),
        &[other.verifying_key()],
        &semver::Version::new(0, 1, 0),
        PLATFORM,
    )
    .unwrap_err();
    assert!(
        matches!(err, CheckError::Feed(FeedError::BadSignature)),
        "{err:?}"
    );
}

#[test]
fn a_download_off_the_allowed_hosts_is_refused() {
    // Signed, but pointing at another server: the policy stops it before
    // anything is fetched.
    let release = publish("0.2.0", release_archive("0.2.0"), |feed, _| {
        feed.platforms.get_mut(PLATFORM.key()).unwrap().url =
            "https://evil.example/Kanso.tar.gz".into();
    });
    let err = check(&release, "0.1.0").unwrap_err();
    assert!(
        matches!(err, CheckError::Feed(FeedError::BadUrl(_))),
        "{err:?}"
    );
}

#[test]
fn offline_and_missing_feeds_are_told_apart() {
    // Nothing listens on the port a dropped listener had.
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let err = kanso_update::check(
        &HttpTransport::new("kanso-test"),
        &LOCAL,
        &format!("http://127.0.0.1:{port}/latest.json"),
        &[SigningKey::from_bytes(&[7; 32]).verifying_key()],
        &semver::Version::new(0, 1, 0),
        PLATFORM,
    )
    .unwrap_err();
    assert!(
        matches!(err, CheckError::Fetch(kanso_update::FetchError::Offline)),
        "{err:?}"
    );

    let base = serve(vec![("/limited".into(), Answer::Status(429))]);
    for (path, expect) in [("/nothing", "not found"), ("/limited", "limiting")] {
        let err = kanso_update::check(
            &HttpTransport::new("kanso-test"),
            &LOCAL,
            &format!("{base}{path}"),
            &[SigningKey::from_bytes(&[7; 32]).verifying_key()],
            &semver::Version::new(0, 1, 0),
            PLATFORM,
        )
        .unwrap_err();
        assert!(err.to_string().contains(expect), "{err}");
    }
}
