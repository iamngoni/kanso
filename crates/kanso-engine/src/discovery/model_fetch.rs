//! Models Kanso downloads once and runs on this device: related notes'
//! embedding model (`related.rs`) and the desktop's voice input speech
//! model. Each comes from a pinned revision of its Hugging Face
//! repository, over HTTPS, into the data folder, and every file is checked
//! against the size and SHA-256 written in the app before it is kept or
//! loaded. The download asks for the model's files and nothing else.
//!
//! A download that stops part way (offline, cancelled, the app quit) keeps
//! what it got in `<file>.part` and carries on from there next time, asking
//! the server for the rest; a server that sends the whole file anyway is
//! read from the start.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write as _};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use sha2::{Digest as _, Sha256};

/// One of a model's files, with the size and SHA-256 it must have.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelFile {
    pub name: &'static str,
    pub size: u64,
    pub sha256: &'static str,
}

/// A model: its files at a pinned revision of a Hugging Face repository,
/// and the folder under the data folder it is kept in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Model {
    pub repository: &'static str,
    pub revision: &'static str,
    pub folder: &'static str,
    pub files: &'static [ModelFile],
}

impl Model {
    /// The model's folder in the data folder `data_dir`.
    pub fn dir(&self, data_dir: &Path) -> PathBuf {
        data_dir.join(self.folder)
    }

    /// The address of one of its files at the pinned revision.
    pub fn file_url(&self, name: &str) -> String {
        format!(
            "https://huggingface.co/{}/resolve/{}/{name}",
            self.repository, self.revision
        )
    }

    /// The size of all its files together.
    pub fn total_size(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }

    /// Its files in `dir` that aren't the pinned ones (missing or different).
    pub fn missing(&self, dir: &Path) -> io::Result<Vec<ModelFile>> {
        let mut out = Vec::new();
        for file in self.files {
            if !verify(&dir.join(file.name), file)? {
                out.push(*file);
            }
        }
        Ok(out)
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Whether the file at `path` is `file`: its size and SHA-256.
pub fn verify(path: &Path, file: &ModelFile) -> io::Result<bool> {
    let meta = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(err),
    };
    if meta.len() != file.size {
        return Ok(false);
    }
    let mut hasher = Sha256::new();
    hash_into(&mut hasher, &mut File::open(path)?, u64::MAX)?;
    Ok(hex(&hasher.finalize()) == file.sha256)
}

/// Feeds up to `limit` bytes of `reader` to `hasher`; how many it read.
fn hash_into(hasher: &mut Sha256, reader: &mut dyn Read, limit: u64) -> io::Result<u64> {
    let mut buf = vec![0u8; 64 * 1024];
    let mut read = 0u64;
    while read < limit {
        let want = buf
            .len()
            .min((limit - read).try_into().unwrap_or(usize::MAX));
        let n = reader.read(&mut buf[..want])?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        read += n as u64;
    }
    Ok(read)
}

/// Why a model couldn't be fetched or loaded.
#[derive(Debug)]
pub enum ModelError {
    Offline,
    Status(u16),
    /// The bytes aren't the pinned file.
    Checksum(&'static str),
    Cancelled,
    Io(io::Error),
    Load(String),
}

impl std::error::Error for ModelError {}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelError::Offline => write!(f, "Couldn't reach the download. Check the connection."),
            ModelError::Status(code) => write!(f, "The download answered {code}."),
            ModelError::Checksum(name) => write!(
                f,
                "The downloaded {name} isn't the expected file, so it wasn't kept."
            ),
            ModelError::Cancelled => write!(f, "Cancelled."),
            ModelError::Io(err) => write!(f, "Couldn't save the model: {err}"),
            ModelError::Load(err) => write!(f, "Couldn't load the model: {err}"),
        }
    }
}

impl From<io::Error> for ModelError {
    fn from(err: io::Error) -> Self {
        ModelError::Io(err)
    }
}

/// A download, opened: its body, and where in the file the body starts
/// (what was asked for, or 0 when the server sent the whole file).
pub struct Opened {
    pub body: Box<dyn Read + Send>,
    pub from: u64,
}

/// Opens a download from byte `from` on. The app's fetches over HTTPS;
/// tests answer from memory.
pub trait Fetch: Send + Sync {
    fn open(&self, url: &str, from: u64) -> Result<Opened, ModelError>;
}

/// HTTPS only, every hop of a redirect too, with no cookies or credentials
/// and no identifying headers beyond a plain user agent. It trusts what
/// Kanso trusts everywhere else (`kanso_tls`): the web's roots on a
/// computer, the phone's own certificate store on iOS and Android.
pub struct HttpFetch {
    agent: ureq::Agent,
}

impl Default for HttpFetch {
    fn default() -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .https_only(true)
            .max_redirects(5)
            .http_status_as_error(false)
            .timeout_connect(Some(Duration::from_secs(20)))
            .timeout_recv_body(Some(Duration::from_secs(60)))
            .user_agent("Kanso")
            .tls_config(kanso_tls::ureq_tls())
            .build()
            .into();
        HttpFetch { agent }
    }
}

impl Fetch for HttpFetch {
    // no-prop-test: no observable oracle offline; the download's rules are
    // tested through `Fetch` from memory.
    fn open(&self, url: &str, from: u64) -> Result<Opened, ModelError> {
        // Before the handshake: an Android app that hasn't handed over its
        // Context gets an error, not a panic inside the verifier.
        kanso_tls::ready_for(url).map_err(|_| ModelError::Offline)?;
        let mut request = self.agent.get(url);
        if from > 0 {
            request = request.header("range", &format!("bytes={from}-"));
        }
        let response = request.call().map_err(|_| ModelError::Offline)?;
        let status = response.status().as_u16();
        let opened = |from| Opened {
            body: Box::new(response.into_body().into_reader()),
            from,
        };
        match status {
            200 => Ok(opened(0)),
            206 if from > 0 => Ok(opened(from)),
            // The part file is longer than the file: start again.
            416 if from > 0 => self.open(url, 0),
            code => Err(ModelError::Status(code)),
        }
    }
}

/// Fetches `model`'s files that `dir` lacks, each kept only when its size
/// and SHA-256 are the pinned ones, carrying on from any part already
/// fetched. `progress` gets the bytes so far and the total.
pub fn download(
    fetch: &dyn Fetch,
    model: &Model,
    dir: &Path,
    progress: &mut dyn FnMut(u64, u64),
    cancel: &AtomicBool,
) -> Result<(), ModelError> {
    fs::create_dir_all(dir)?;
    let todo = model.missing(dir)?;
    let total: u64 = todo.iter().map(|f| f.size).sum();
    let mut done = 0u64;
    progress(0, total);
    for file in todo {
        let to = dir.join(file.name);
        let part = dir.join(format!("{}.part", file.name));
        let result = fetch_file(
            fetch,
            &model.file_url(file.name),
            &file,
            &part,
            &mut |got| progress(done + got, total),
            cancel,
        );
        match result {
            Ok(()) => fs::rename(&part, &to)?,
            // What arrived is kept to carry on from.
            Err(err @ (ModelError::Offline | ModelError::Cancelled | ModelError::Status(_))) => {
                return Err(err);
            }
            Err(err) => {
                fs::remove_file(&part).ok();
                return Err(err);
            }
        }
        done += file.size;
    }
    Ok(())
}

/// Fetches one file into `part`, carrying on from what `part` holds, and
/// checks it whole. `progress` gets this file's bytes so far.
fn fetch_file(
    fetch: &dyn Fetch,
    url: &str,
    file: &ModelFile,
    part: &Path,
    progress: &mut dyn FnMut(u64),
    cancel: &AtomicBool,
) -> Result<(), ModelError> {
    let held = fs::metadata(part).map(|m| m.len()).unwrap_or(0);
    let held = if held < file.size { held } else { 0 };
    let opened = fetch.open(url, held)?;
    let mut hasher = Sha256::new();
    let mut out = if opened.from > 0 && opened.from == held {
        // Carry on: what is held goes into the hash first.
        let got = hash_into(&mut hasher, &mut File::open(part)?, held)?;
        if got != held {
            return Err(ModelError::Checksum(file.name));
        }
        OpenOptions::new().append(true).open(part)?
    } else {
        File::create(part)?
    };
    let mut got = if opened.from > 0 { held } else { 0 };
    progress(got);
    let mut body = opened.body;
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        if cancel.load(Ordering::Relaxed) {
            out.sync_all().ok();
            return Err(ModelError::Cancelled);
        }
        let n = match body.read(&mut buf) {
            Ok(n) => n,
            Err(_) => {
                out.sync_all().ok();
                return Err(ModelError::Offline);
            }
        };
        if n == 0 {
            break;
        }
        got += n as u64;
        if got > file.size {
            return Err(ModelError::Checksum(file.name));
        }
        hasher.update(&buf[..n]);
        out.write_all(&buf[..n])?;
        progress(got);
    }
    out.sync_all()?;
    if got != file.size || hex(&hasher.finalize()) != file.sha256 {
        return Err(ModelError::Checksum(file.name));
    }
    Ok(())
}

/// Deletes a model's folder and everything in it, if it is there.
pub fn remove(dir: &Path) -> io::Result<()> {
    match fs::remove_dir_all(dir) {
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

/// How far a download is, as a whole percentage.
pub fn percent(done: u64, total: u64) -> u32 {
    if total == 0 {
        return 100;
    }
    ((done.min(total) as u128 * 100) / total as u128) as u32
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Answers from memory, honouring a range when `ranges` is set, and
    /// failing a body after `cut` bytes when set, counting what was asked.
    pub(crate) struct Memory {
        pub files: HashMap<String, Vec<u8>>,
        pub ranges: bool,
        pub cut: Option<usize>,
        pub asked: Mutex<Vec<(String, u64)>>,
    }

    impl Memory {
        pub fn new(files: HashMap<String, Vec<u8>>) -> Memory {
            Memory {
                files,
                ranges: true,
                cut: None,
                asked: Mutex::new(Vec::new()),
            }
        }
    }

    /// A body that fails after its bytes run out, like a dropped connection.
    struct Dropped(io::Cursor<Vec<u8>>);

    impl Read for Dropped {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            match self.0.read(buf)? {
                0 => Err(io::Error::new(io::ErrorKind::ConnectionReset, "dropped")),
                n => Ok(n),
            }
        }
    }

    impl Fetch for Memory {
        fn open(&self, url: &str, from: u64) -> Result<Opened, ModelError> {
            self.asked.lock().unwrap().push((url.to_owned(), from));
            let bytes = self.files.get(url).ok_or(ModelError::Status(404))?;
            let from = if self.ranges { from } else { 0 };
            let rest = bytes[(from as usize).min(bytes.len())..].to_vec();
            let body: Box<dyn Read + Send> = match self.cut {
                Some(cut) if cut < rest.len() => {
                    Box::new(Dropped(io::Cursor::new(rest[..cut].to_vec())))
                }
                _ => Box::new(io::Cursor::new(rest)),
            };
            Ok(Opened { body, from })
        }
    }

    const ABC: ModelFile = ModelFile {
        name: "f",
        size: 3,
        // SHA-256 of "abc".
        sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    };

    #[test]
    fn verify_checks_size_and_hash() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("f");
        assert!(!verify(&path, &ABC).unwrap());
        fs::write(&path, b"abc").unwrap();
        assert!(verify(&path, &ABC).unwrap());
        fs::write(&path, b"abd").unwrap();
        assert!(!verify(&path, &ABC).unwrap());
        fs::write(&path, b"abcd").unwrap();
        assert!(!verify(&path, &ABC).unwrap());
    }

    #[test]
    fn remove_takes_the_folder_and_minds_none() {
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("models/m");
        fs::create_dir_all(&model).unwrap();
        fs::write(model.join("f"), b"abc").unwrap();
        remove(&model).unwrap();
        assert!(!model.exists());
        remove(&model).unwrap();
    }

    /// A pinned file of `len` bytes from `seed`, for the download tests.
    fn file_of(len: usize, seed: u8) -> (Vec<u8>, ModelFile) {
        let bytes: Vec<u8> = (0..len)
            .map(|i| (i as u8).wrapping_mul(31).wrapping_add(seed))
            .collect();
        let sha = hex(&Sha256::digest(&bytes));
        let file = ModelFile {
            name: "weights.bin",
            size: len as u64,
            sha256: Box::leak(sha.into_boxed_str()),
        };
        (bytes, file)
    }

    fn one_file_model(file: ModelFile) -> Model {
        Model {
            repository: "o/m",
            revision: "r",
            folder: "models/m",
            files: Box::leak(vec![file].into_boxed_slice()),
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 128, ..ProptestConfig::default() })]

        /// However the download is cut off, carrying on finishes with the
        /// pinned file, whether or not the server honours the range, and
        /// progress never goes backwards past where it carried on from.
        #[test]
        fn prop_a_cut_download_carries_on_to_the_pinned_file(
            len in 1usize..5_000,
            seed: u8,
            cut_at in 0usize..5_000,
            ranges: bool,
        ) {
            let (bytes, file) = file_of(len, seed);
            let model = one_file_model(file);
            let dir = tempfile::tempdir().unwrap();
            let cancel = AtomicBool::new(false);
            let files: HashMap<_, _> = [(model.file_url(file.name), bytes.clone())].into();
            let mut first = Memory::new(files.clone());
            first.cut = Some(cut_at % len);
            let err = download(&first, &model, dir.path(), &mut |_, _| {}, &cancel).unwrap_err();
            prop_assert!(matches!(err, ModelError::Offline));
            let held = fs::metadata(dir.path().join("weights.bin.part")).map(|m| m.len()).unwrap_or(0);
            prop_assert_eq!(held, (cut_at % len) as u64);

            let mut second = Memory::new(files);
            second.ranges = ranges;
            let mut seen = Vec::new();
            download(&second, &model, dir.path(), &mut |done, total| seen.push((done, total)), &cancel).unwrap();
            prop_assert!(verify(&dir.path().join(file.name), &file).unwrap());
            prop_assert!(!dir.path().join("weights.bin.part").exists());
            prop_assert_eq!(second.asked.lock().unwrap()[0].1, held);
            prop_assert_eq!(seen.last().copied(), Some((len as u64, len as u64)));
            for pair in seen.windows(2).skip(1) {
                prop_assert!(pair[1].0 >= pair[0].0);
            }
        }

        /// The progress shown never goes past 100% or backwards.
        #[test]
        fn prop_percent_is_monotonic_and_bounded(done in 0u64..10_000_000, more in 0u64..10_000_000, total in 0u64..10_000_000) {
            let a = percent(done, total);
            prop_assert!(a <= 100);
            prop_assert!(percent(done + more, total) >= a);
        }
    }

    /// Bytes that aren't the pinned file are never kept, nor is a body
    /// longer than the file, and a part file that is wrong is dropped.
    #[test]
    fn a_tampered_download_is_refused() {
        let (bytes, file) = file_of(100, 7);
        let model = one_file_model(file);
        let url = model.file_url(file.name);
        let cancel = AtomicBool::new(false);
        let dir = tempfile::tempdir().unwrap();
        let wrong = Memory::new([(url.clone(), vec![b'x'; 100])].into());
        assert!(matches!(
            download(&wrong, &model, dir.path(), &mut |_, _| {}, &cancel),
            Err(ModelError::Checksum("weights.bin"))
        ));
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
        let long = Memory::new([(url.clone(), [bytes.clone(), vec![0; 10]].concat())].into());
        assert!(matches!(
            download(&long, &model, dir.path(), &mut |_, _| {}, &cancel),
            Err(ModelError::Checksum(_))
        ));
        // A part file with wrong bytes in it fails the whole check once
        // carried on from, and is dropped, so the next try starts clean.
        fs::write(dir.path().join("weights.bin.part"), vec![b'q'; 40]).unwrap();
        let right = Memory::new([(url, bytes)].into());
        assert!(matches!(
            download(&right, &model, dir.path(), &mut |_, _| {}, &cancel),
            Err(ModelError::Checksum(_))
        ));
        download(&right, &model, dir.path(), &mut |_, _| {}, &cancel).unwrap();
        assert!(verify(&dir.path().join(file.name), &file).unwrap());
        // Offline from the start: a clear error, nothing kept.
        let empty = tempfile::tempdir().unwrap();
        let nothing = Memory::new(HashMap::new());
        assert!(matches!(
            download(&nothing, &model, empty.path(), &mut |_, _| {}, &cancel),
            Err(ModelError::Status(404))
        ));
    }
}
