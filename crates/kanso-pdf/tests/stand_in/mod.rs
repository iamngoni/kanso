//! Writing a stand-in worker script that can be run at once.

#![allow(dead_code)]

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

/// Writes the shell script `body` to `path`, executable.
///
/// On Linux, running a file that some process still has open for writing
/// fails with "Text file busy". Tests run on many threads, and a thread that
/// forks while this one has the file open hands its child a copy of that
/// descriptor until the child execs. So this process never opens the script
/// for writing: it writes the text beside it, and `cp`, a separate process
/// that has exited by the time the script runs, makes the script.
pub fn write_script(path: &Path, body: &str) {
    let text = path.with_extension("txt");
    std::fs::write(&text, format!("#!/bin/sh\n{body}\n")).unwrap();
    let copied = Command::new("cp").arg(&text).arg(path).status().unwrap();
    assert!(copied.success(), "cp failed for {path:?}");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}
