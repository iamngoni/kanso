//! Writes the hostile-PDF corpus, or the protocol frames the fuzz target
//! starts from, to a folder (issue #43, step 9):
//!
//! ```sh
//! cargo run -p kanso-pdf --features test-support --example pdf_corpus -- pdfs  <dir>
//! cargo run -p kanso-pdf --features test-support --example pdf_corpus -- frames <dir>
//! ```
//!
//! `scripts/pdf-corpus.sh` wraps both. Nothing is downloaded: every file is
//! built in code (`kanso_pdf::hostile`).

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use kanso_pdf::hostile::{Canary, corpus};
use kanso_pdf::protocol::{ErrorKind, Request, Response, write_frame};
use serde::Serialize;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [what, dir] = args.as_slice() else {
        eprintln!("usage: pdf_corpus (pdfs|frames) <folder>");
        return ExitCode::from(2);
    };
    let dir = Path::new(dir);
    let result = match what.as_str() {
        "pdfs" => write_pdfs(dir),
        "frames" => write_frames(dir),
        _ => {
            eprintln!("usage: pdf_corpus (pdfs|frames) <folder>");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(count) => {
            println!("{count} files in {}", dir.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("pdf_corpus: {e}");
            ExitCode::FAILURE
        }
    }
}

fn write_pdfs(dir: &Path) -> std::io::Result<usize> {
    fs::create_dir_all(dir)?;
    let samples = corpus(&Canary::unreachable());
    for sample in &samples {
        fs::write(dir.join(format!("{}.pdf", sample.name)), &sample.bytes)?;
    }
    Ok(samples.len())
}

/// Valid frames in both directions, and the corpus's small files as the
/// document an `Open` carries: the fuzzer mutates from messages the
/// decoder accepts rather than from nothing.
fn write_frames(dir: &Path) -> std::io::Result<usize> {
    fs::create_dir_all(dir)?;
    let mut count = 0;
    let mut put = |name: String, frame: Vec<u8>| -> std::io::Result<()> {
        count += 1;
        fs::write(dir.join(name), frame)
    };
    put("close".into(), frame(&Request::Close, &[]))?;
    put(
        "render".into(),
        frame(
            &Request::Render {
                page: 3,
                width_px: 1224,
            },
            &[],
        ),
    )?;
    put(
        "open-password".into(),
        frame(
            &Request::Open {
                password: Some("secret".into()),
            },
            b"%PDF-1.7\n",
        ),
    )?;
    put(
        "opened".into(),
        frame(
            &Response::Opened {
                pages: 2,
                sizes: vec![[612.0, 792.0], [595.0, 842.0]],
                encrypted: false,
            },
            &[],
        ),
    )?;
    put(
        "page".into(),
        frame(
            &Response::Page {
                page: 0,
                width: 2,
                height: 2,
            },
            &[0u8; 16],
        ),
    )?;
    put("closed".into(), frame(&Response::Closed, &[]))?;
    put(
        "error".into(),
        frame(
            &Response::Error {
                kind: ErrorKind::PageTooLarge,
                message: "x".into(),
            },
            &[],
        ),
    )?;
    for sample in corpus(&Canary::unreachable())
        .into_iter()
        .filter(|s| s.bytes.len() <= 64 * 1024)
    {
        put(
            format!("open-{}", sample.name),
            frame(&Request::Open { password: None }, &sample.bytes),
        )?;
    }
    Ok(count)
}

/// One frame, as the protocol writes it.
fn frame<M: Serialize>(message: &M, blob: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    write_frame(&mut out, message, blob).expect("a seed frame fits");
    out
}
