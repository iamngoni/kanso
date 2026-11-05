//! A corpus of malformed and hostile PDFs (issue #43, step 9), built in code
//! so no binary lives in the repository and nothing is downloaded from
//! anywhere: truncated and looping files, absurd sizes, deep nesting, every
//! kind of action a PDF can carry, decompression bombs, junk in every
//! decoder PDFium has. Behind the `test-support` feature.
//!
//! Each [`Sample`] says what it is and what a safe reader does with it
//! ([`Expect`]). `tests/hostile.rs` runs every one through the real worker;
//! `cargo run -p kanso-pdf --features test-support --example pdf_corpus`
//! (or `scripts/pdf-corpus.sh`) writes them to a folder, and writes the
//! protocol frames the fuzz target starts from.

use std::path::PathBuf;

use crate::fixtures::{encrypted_pdf, sample_pdf};
use crate::protocol::{ErrorKind, MAX_PAGES};

/// What the actions in the corpus try to reach. The test watches both: a
/// server nothing may connect to, and a file nothing may create.
#[derive(Clone, Debug)]
pub struct Canary {
    /// A URL the document's URI, JavaScript and form-submit actions point at.
    pub url: String,
    /// A file the launch action and the embedded script would create.
    pub marker: PathBuf,
}

impl Canary {
    /// For writing the corpus out: an address nothing listens on.
    pub fn unreachable() -> Canary {
        Canary {
            url: "http://127.0.0.1:9/kanso-canary".to_owned(),
            marker: std::env::temp_dir().join("kanso-pdf-canary"),
        }
    }
}

/// What a safe reader does with a sample.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expect {
    /// It opens and every page it draws is within the caps.
    Opens,
    /// The open is refused, with one of these kinds.
    Refused(&'static [ErrorKind]),
    /// It may open (and then draw within the caps) or be refused; what it
    /// may not do is crash the worker, hang past the deadline or break a cap.
    OpensOrRefused,
    /// It is a time bomb: drawing a page outlasts the deadline, and the
    /// worker is killed. (If PDFium ever learns to refuse it quickly, it may
    /// also be refused or drawn.)
    TimesOut,
}

/// One file of the corpus.
#[derive(Clone, Debug)]
pub struct Sample {
    pub name: &'static str,
    pub what: &'static str,
    pub bytes: Vec<u8>,
    pub expect: Expect,
}

/// A pseudo-random byte stream, the same every run.
fn junk(len: usize, seed: u64) -> Vec<u8> {
    let mut x = seed | 1;
    (0..len)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            (x >> 24) as u8
        })
        .collect()
}

/// A stream object: the dictionary entries, then the bytes.
fn stream(entries: &str, data: &[u8]) -> Vec<u8> {
    let mut out = format!("<< /Length {} {entries} >>\nstream\n", data.len()).into_bytes();
    out.extend_from_slice(data);
    out.extend_from_slice(b"\nendstream");
    out
}

/// A one-page document with the pieces a sample changes. Objects 1 to 5 are
/// the catalog, the page tree, a font, the page and its content; `extra`
/// objects are numbered from 6.
struct Spec {
    media_box: String,
    catalog: String,
    page: String,
    resources: String,
    content: Vec<u8>,
    trailer: String,
    extra: Vec<Vec<u8>>,
}

impl Default for Spec {
    fn default() -> Self {
        Spec {
            media_box: "[0 0 612 792]".to_owned(),
            catalog: String::new(),
            page: String::new(),
            resources: String::new(),
            content: b"BT /F1 24 Tf 72 700 Td (Hostile) Tj ET".to_vec(),
            trailer: String::new(),
            extra: Vec::new(),
        }
    }
}

impl Spec {
    fn build(&self) -> Vec<u8> {
        let mut objects: Vec<Vec<u8>> = vec![
            format!("<< /Type /Catalog /Pages 2 0 R {} >>", self.catalog).into_bytes(),
            b"<< /Type /Pages /Kids [4 0 R] /Count 1 >>".to_vec(),
            b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox {} /Resources << /Font << /F1 3 0 R >> {} >> /Contents 5 0 R {} >>",
                self.media_box, self.resources, self.page
            )
            .into_bytes(),
            stream("", &self.content),
        ];
        objects.extend(self.extra.iter().cloned());
        assemble(&objects, &self.trailer)
    }
}

/// Numbered objects (from 1) with a correct cross-reference table.
fn assemble(objects: &[Vec<u8>], trailer_extra: &str) -> Vec<u8> {
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    out.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for offset in offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R {trailer_extra} >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

/// A PDF string literal's content with the characters that end it escaped.
fn literal(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

fn doc(spec: Spec) -> Vec<u8> {
    spec.build()
}

/// A single `/Link` annotation whose action is `action`, over the page.
fn link_page(action: &str) -> Spec {
    Spec {
        page: "/Annots [6 0 R]".to_owned(),
        extra: vec![
            format!(
                "<< /Type /Annot /Subtype /Link /Rect [0 0 612 792] /Border [0 0 0] /A {action} >>"
            )
            .into_bytes(),
        ],
        ..Spec::default()
    }
}

/// A document of `pages` pages that share one page object: a few hundred KB
/// that claim a hundred thousand pages.
fn shared_page_bomb(pages: usize) -> Vec<u8> {
    let kids = "4 0 R ".repeat(pages);
    let objects = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        format!("<< /Type /Pages /Kids [{kids}] /Count {pages} >>").into_bytes(),
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R >> >> /Contents 5 0 R >>".to_vec(),
        stream("", b"BT /F1 24 Tf 72 700 Td (Shared) Tj ET"),
    ];
    assemble(&objects, "")
}

/// A page tree `depth` levels deep, one `/Pages` node inside the next.
fn deep_page_tree(depth: usize) -> Vec<u8> {
    // 1 catalog, 2.. the nodes, then the leaf page, a font and a content stream.
    let leaf = depth + 2;
    let font = leaf + 1;
    let content = leaf + 2;
    let mut objects = vec![b"<< /Type /Catalog /Pages 2 0 R >>".to_vec()];
    for level in 0..depth {
        let me = 2 + level;
        let child = me + 1;
        let parent = if level == 0 {
            String::new()
        } else {
            format!("/Parent {} 0 R", me - 1)
        };
        objects
            .push(format!("<< /Type /Pages /Kids [{child} 0 R] /Count 1 {parent} >>").into_bytes());
    }
    objects.push(
        format!(
            "<< /Type /Page /Parent {} 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 {font} 0 R >> >> /Contents {content} 0 R >>",
            leaf - 1
        )
        .into_bytes(),
    );
    objects.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec());
    objects.push(stream("", b"BT /F1 24 Tf 72 700 Td (Deep) Tj ET"));
    assemble(&objects, "")
}

/// Two form XObjects per level, each drawing the next level twice: `levels`
/// deep that is 2^`levels` draws from a file of a few kilobytes.
fn form_bomb(levels: usize) -> Vec<u8> {
    // Forms are objects 6..; the last one draws a rectangle.
    let mut extra = Vec::new();
    for level in 0..levels {
        let name = level + 1;
        let content = if level + 1 == levels {
            "0 0 10 10 re f".to_owned()
        } else {
            format!("/X{} Do /X{} Do", name + 1, name + 1)
        };
        let next = if level + 1 == levels {
            String::new()
        } else {
            format!("/XObject << /X{} {} 0 R >>", name + 1, 6 + level + 1)
        };
        extra.push(stream(
            &format!("/Type /XObject /Subtype /Form /BBox [0 0 612 792] /Resources << {next} >>"),
            content.as_bytes(),
        ));
    }
    doc(Spec {
        resources: "/XObject << /X1 6 0 R >>".to_owned(),
        content: b"/X1 Do".to_vec(),
        extra,
        ..Spec::default()
    })
}

/// A form that draws itself.
fn form_self_reference() -> Vec<u8> {
    doc(Spec {
        resources: "/XObject << /X1 6 0 R >>".to_owned(),
        content: b"/X1 Do".to_vec(),
        extra: vec![stream(
            "/Type /XObject /Subtype /Form /BBox [0 0 612 792] /Resources << /XObject << /X1 6 0 R >> >>",
            b"0 0 10 10 re f /X1 Do",
        )],
        ..Spec::default()
    })
}

/// Run-length coding twice over: a 2-byte pair expands to 128 bytes, so
/// `pairs` pairs expand to 128 * 64 * pairs bytes.
fn double_run_length(pairs: usize) -> Vec<u8> {
    [0x81u8, 0x81].repeat(pairs)
}

/// An image drawn over the page.
fn image_page(image: Vec<u8>) -> Vec<u8> {
    doc(Spec {
        resources: "/XObject << /Im1 6 0 R >>".to_owned(),
        content: b"q 612 0 0 792 0 0 cm /Im1 Do Q".to_vec(),
        extra: vec![image],
        ..Spec::default()
    })
}

/// An image whose data goes through `filter` and is not what the filter reads.
fn junk_image(filter: &str, extra_entries: &str, seed: u64) -> Vec<u8> {
    image_page(stream(
        &format!(
            "/Type /XObject /Subtype /Image /Width 64 /Height 64 /BitsPerComponent 8 /ColorSpace /DeviceGray /Filter /{filter} {extra_entries}"
        ),
        &junk(512, seed),
    ))
}

/// A document whose catalog or page ties `action` to opening, with a
/// matching link annotation and page action.
fn actions(action: &str) -> Vec<u8> {
    let mut spec = link_page(action);
    spec.catalog = format!("/OpenAction {action}");
    spec.page = format!("/Annots [6 0 R] /AA << /O {action} /C {action} >>");
    doc(spec)
}

/// The corpus. `canary` is what the actions try to reach.
pub fn corpus(canary: &Canary) -> Vec<Sample> {
    let url = literal(&canary.url);
    let marker = literal(&canary.marker.to_string_lossy());
    let valid = sample_pdf(5);
    let cut = |at: usize| valid[..at.min(valid.len())].to_vec();

    use Expect::{Opens, OpensOrRefused, Refused, TimesOut};
    const UNREADABLE: &[ErrorKind] = &[ErrorKind::Unreadable];
    const TOO_MANY: &[ErrorKind] = &[ErrorKind::TooManyPages];
    const TOO_LARGE: &[ErrorKind] = &[ErrorKind::PageTooLarge];
    let mut samples: Vec<Sample> = Vec::new();
    let mut add = |name, what, bytes: Vec<u8>, expect| {
        samples.push(Sample {
            name,
            what,
            bytes,
            expect,
        });
    };

    // Not PDFs at all -----------------------------------------------------
    add("empty", "zero bytes", Vec::new(), Refused(UNREADABLE));
    add(
        "junk-random",
        "4 KiB of noise",
        junk(4096, 7),
        Refused(UNREADABLE),
    );
    add(
        "junk-after-header",
        "a PDF header, then noise",
        [b"%PDF-1.7\n".to_vec(), junk(2048, 11)].concat(),
        Refused(UNREADABLE),
    );
    add(
        "plain-text",
        "a text file",
        b"This is not a PDF.\n".to_vec(),
        Refused(UNREADABLE),
    );
    add(
        "html-with-script",
        "a web page with a script",
        b"<html><script>alert(document.cookie)</script></html>".to_vec(),
        Refused(UNREADABLE),
    );
    add(
        "nul-bytes",
        "a mebibyte of zeros",
        vec![0u8; 1 << 20],
        Refused(UNREADABLE),
    );
    add(
        "junk-before-header",
        "900 bytes of noise, then a PDF",
        [junk(900, 13), valid.clone()].concat(),
        OpensOrRefused,
    );

    // Truncated -----------------------------------------------------------
    add(
        "truncated-header",
        "cut inside the header",
        cut(6),
        Refused(UNREADABLE),
    );
    add(
        "truncated-half",
        "cut in the middle of the objects",
        cut(valid.len() / 2),
        OpensOrRefused,
    );
    add(
        "truncated-in-xref",
        "cut inside the cross-reference table",
        cut(valid.len() - 400),
        OpensOrRefused,
    );
    add(
        "truncated-no-eof",
        "the last few bytes missing",
        cut(valid.len() - 7),
        OpensOrRefused,
    );

    // Cross-reference and structure ----------------------------------------
    add(
        "xref-prev-self-loop",
        "a cross-reference table whose /Prev is itself",
        {
            let text = String::from_utf8_lossy(&valid).into_owned();
            let at = text.rfind("startxref\n").unwrap() + "startxref\n".len();
            let xref: usize = text[at..].lines().next().unwrap().parse().unwrap();
            text.replace("/Root 1 0 R", &format!("/Root 1 0 R /Prev {xref}"))
                .into_bytes()
        },
        OpensOrRefused,
    );
    add(
        "xref-prev-cycle",
        "two sections that name each other",
        xref_cycle(),
        OpensOrRefused,
    );
    add(
        "xref-size-huge",
        "a table that declares four billion entries",
        String::from_utf8_lossy(&sample_pdf(1))
            .replace("xref\n0 7", "xref\n0 4294967295")
            .replace("/Size 7", "/Size 4294967295")
            .into_bytes(),
        OpensOrRefused,
    );
    add(
        "xref-offsets-zero",
        "every offset in the table is zero",
        {
            let text = String::from_utf8_lossy(&sample_pdf(2)).into_owned();
            let mut out = String::new();
            for line in text.lines() {
                if line.ends_with(" 00000 n ") {
                    out.push_str("0000000000 00000 n \n");
                } else {
                    out.push_str(line);
                    out.push('\n');
                }
            }
            out.into_bytes()
        },
        OpensOrRefused,
    );
    add(
        "startxref-overflow",
        "startxref names a number that does not fit",
        {
            let text = String::from_utf8_lossy(&sample_pdf(1)).into_owned();
            let cut = text.rfind("startxref").unwrap();
            format!(
                "{}startxref\n99999999999999999999999\n%%EOF\n",
                &text[..cut]
            )
            .into_bytes()
        },
        OpensOrRefused,
    );
    add(
        "object-number-huge",
        "an object numbered 4294967295",
        {
            let text = String::from_utf8_lossy(&sample_pdf(1))
                .replace("\n6 0 obj", "\n4294967295 0 obj")
                .replace("/Contents 6 0 R", "/Contents 4294967295 0 R");
            text.into_bytes()
        },
        OpensOrRefused,
    );
    add(
        "stream-length-lies",
        "a content stream that says it is two gigabytes",
        {
            let text = String::from_utf8_lossy(&sample_pdf(1)).into_owned();
            let at = text.find("<< /Length ").unwrap() + "<< /Length ".len();
            let end = at + text[at..].find(' ').unwrap();
            format!("{}2000000000{}", &text[..at], &text[end..]).into_bytes()
        },
        OpensOrRefused,
    );
    add(
        "pages-tree-cycle",
        "a page tree whose only kid is itself",
        String::from_utf8_lossy(&sample_pdf(1))
            .replace("/Kids [5 0 R]", "/Kids [2 0 R]")
            .into_bytes(),
        OpensOrRefused,
    );
    add(
        "pages-count-lie",
        "/Count claims two billion pages; there is one",
        String::from_utf8_lossy(&sample_pdf(1))
            .replace("/Count 1", "/Count 2000000000")
            .into_bytes(),
        OpensOrRefused,
    );
    add(
        "pages-shared-kids",
        "one page listed 100,000 times in the page tree",
        shared_page_bomb(100_000),
        OpensOrRefused,
    );
    add(
        "pages-tree-5000-deep",
        "5,000 page-tree nodes, one inside the next",
        deep_page_tree(5_000),
        OpensOrRefused,
    );

    // Page counts ---------------------------------------------------------
    add(
        "pages-at-limit",
        "exactly the page limit",
        sample_pdf(MAX_PAGES as usize),
        Opens,
    );
    add(
        "pages-over-limit",
        "one page over the limit",
        sample_pdf(MAX_PAGES as usize + 1),
        Refused(TOO_MANY),
    );
    add(
        "pages-4000",
        "twice the limit",
        sample_pdf(2 * MAX_PAGES as usize),
        Refused(TOO_MANY),
    );

    // Declared dimensions ---------------------------------------------------
    // PDFium reports the first two as declared, so the size check refuses
    // them; the others it brings to a size of its own, and the render caps
    // hold either way.
    for (name, what, media, expect) in [
        (
            "mediabox-1e9",
            "a page a billion points square",
            "[0 0 1000000000 1000000000]",
            Refused(TOO_LARGE),
        ),
        (
            "mediabox-sliver",
            "a page one point by a hundred million",
            "[0 0 1 100000000]",
            Refused(TOO_LARGE),
        ),
        (
            "mediabox-zero",
            "a page of no size",
            "[0 0 0 0]",
            OpensOrRefused,
        ),
        (
            "mediabox-negative",
            "corners the wrong way round",
            "[500 700 -500 -700]",
            OpensOrRefused,
        ),
        (
            "mediabox-garbage",
            "a box that is not numbers",
            "[(a) /b null 1e999]",
            OpensOrRefused,
        ),
        (
            "mediabox-float-max",
            "a box at the edge of a float",
            "[-3.4e38 -3.4e38 3.4e38 3.4e38]",
            OpensOrRefused,
        ),
    ] {
        add(
            name,
            what,
            doc(Spec {
                media_box: media.to_owned(),
                ..Spec::default()
            }),
            expect,
        );
    }
    add(
        "user-unit-huge",
        "a user unit of a million on a Letter page",
        doc(Spec {
            page: "/UserUnit 1000000".to_owned(),
            ..Spec::default()
        }),
        OpensOrRefused,
    );
    add(
        "rotate-odd",
        "a rotation that is not a right angle",
        doc(Spec {
            page: "/Rotate 45".to_owned(),
            ..Spec::default()
        }),
        OpensOrRefused,
    );
    add(
        "rotate-huge",
        "a rotation of two billion degrees",
        doc(Spec {
            page: "/Rotate 2000000000".to_owned(),
            ..Spec::default()
        }),
        OpensOrRefused,
    );

    // Nesting and recursion -------------------------------------------------
    add(
        "nesting-arrays",
        "an array nested 100,000 deep",
        doc(Spec {
            page: format!("/Deep {}{}", "[".repeat(100_000), "]".repeat(100_000)),
            ..Spec::default()
        }),
        OpensOrRefused,
    );
    add(
        "nesting-dictionaries",
        "a dictionary nested 50,000 deep",
        doc(Spec {
            page: format!("/Deep {}1{}", "<< /A ".repeat(50_000), " >>".repeat(50_000)),
            ..Spec::default()
        }),
        OpensOrRefused,
    );
    add(
        "graphics-state-stack",
        "a million `q` operators with no `Q`",
        doc(Spec {
            content: b"q ".repeat(1_000_000),
            ..Spec::default()
        }),
        OpensOrRefused,
    );
    add(
        "form-draws-itself",
        "a form XObject that draws itself",
        form_self_reference(),
        OpensOrRefused,
    );
    add(
        "form-bomb",
        "forms drawing forms twice each, 40 levels: 2^40 draws",
        form_bomb(40),
        TimesOut,
    );
    add(
        "dash-bomb",
        "a dash pattern a millionth of a point long on a long line",
        doc(Spec {
            content: b"[0.000001] 0 d 1 w 0 0 m 612 792 l S".to_vec(),
            ..Spec::default()
        }),
        OpensOrRefused,
    );
    add(
        "content-numbers",
        "matrices and widths at the edge of a float",
        doc(Spec {
            content: b"1e38 0 0 1e38 0 0 cm 1e308 w 0 0 m 1e38 1e38 l S 0 0 1e38 1e38 re f"
                .to_vec(),
            ..Spec::default()
        }),
        OpensOrRefused,
    );
    add(
        "content-noise",
        "a page whose content is noise",
        doc(Spec {
            content: junk(8192, 17),
            ..Spec::default()
        }),
        OpensOrRefused,
    );

    // Actions: valid files that ask the reader to do things. They must open
    // and draw, or the canary checks would pass without PDFium ever having
    // read an action. ------------------------------------------------------
    let javascript = format!(
        "<< /S /JavaScript /JS (app.launchURL\\(\"{url}\", true\\); this.submitForm\\(\"{url}\"\\);) >>"
    );
    add(
        "javascript-actions",
        "JavaScript on open, on the page, on a link and in the name tree",
        {
            // Object 6 is the link; 7 is the action the document-level
            // JavaScript name tree runs on open (a name tree's values are
            // actions, so it must not point at the link).
            let mut spec = link_page(&javascript);
            spec.catalog = format!(
                "/Names << /JavaScript << /Names [(boot) 7 0 R] >> >> /OpenAction {javascript}"
            );
            spec.page = format!("/Annots [6 0 R] /AA << /O {javascript} /C {javascript} >>");
            spec.extra.push(javascript.clone().into_bytes());
            doc(spec)
        },
        Opens,
    );
    add(
        "javascript-form-fields",
        "a text field that calculates and validates with JavaScript",
        doc(Spec {
            catalog: "/AcroForm << /Fields [6 0 R] /NeedAppearances true /CO [6 0 R] >>"
                .to_owned(),
            page: "/Annots [6 0 R]".to_owned(),
            extra: vec![
                format!(
                    "<< /Type /Annot /Subtype /Widget /FT /Tx /T (total) /V (1) /Rect [72 600 300 640] /P 4 0 R /AA << /C << /S /JavaScript /JS (app.launchURL\\(\"{url}\"\\);) >> /V << /S /JavaScript /JS (event.rc = true;) >> /K << /S /JavaScript /JS (app.alert\\(1\\);) >> >> >>"
                )
                .into_bytes(),
            ],
            ..Spec::default()
        }),
        Opens,
    );
    add(
        "launch-action",
        "a launch action, on open and on a link, naming a program",
        {
            let launch = format!(
                "<< /S /Launch /F (/usr/bin/touch) /Win << /F (cmd.exe) /P (/c echo > \"{marker}\") >> /Unix << /F (/usr/bin/touch) /P ({marker}) >> >>"
            );
            actions(&launch)
        },
        Opens,
    );
    add(
        "uri-action",
        "a URI action, on open and on a link, naming a server",
        actions(&format!("<< /S /URI /URI ({url}) >>")),
        Opens,
    );
    add(
        "submit-form-action",
        "a form-submit action posting to a server",
        actions(&format!(
            "<< /S /SubmitForm /F << /FS /URL /F ({url}) >> /Flags 4 >>"
        )),
        Opens,
    );
    add(
        "remote-file-actions",
        "go-to-remote, import-data and named actions naming local files",
        actions(
            "<< /S /GoToR /F << /FS /URL /F (file:///etc/passwd) >> /D [0 /Fit] /NewWindow true /Next << /S /ImportData /F (/etc/passwd) /Next << /S /Named /N /Print >> >> >>",
        ),
        Opens,
    );
    add(
        "embedded-file",
        "an embedded script, in the name tree and as a file attachment",
        doc(Spec {
            catalog: "/Names << /EmbeddedFiles << /Names [(run.sh) 7 0 R] >> >> /AF [7 0 R]"
                .to_owned(),
            page: "/Annots [6 0 R]".to_owned(),
            extra: vec![
                b"<< /Type /Annot /Subtype /FileAttachment /Rect [10 10 30 30] /Name /Paperclip /FS 7 0 R >>"
                    .to_vec(),
                b"<< /Type /Filespec /F (run.sh) /UF (run.sh) /AFRelationship /Unspecified /EF << /F 8 0 R /UF 8 0 R >> >>"
                    .to_vec(),
                stream(
                    "/Type /EmbeddedFile /Subtype /application#2Fx-sh",
                    format!("#!/bin/sh\ntouch '{}'\n", canary.marker.display()).as_bytes(),
                ),
            ],
            ..Spec::default()
        }),
        Opens,
    );
    add(
        "xfa-form",
        "an XFA form with a script",
        doc(Spec {
            catalog: "/AcroForm << /Fields [] /XFA [(template) 6 0 R] >>".to_owned(),
            extra: vec![stream(
                "",
                format!(
                    "<xdp:xdp xmlns:xdp=\"http://ns.adobe.com/xdp/\"><template><subform><event activity=\"initialize\"><script contentType=\"application/x-javascript\">xfa.host.gotoURL(\"{}\");</script></event></subform></template></xdp:xdp>",
                    canary.url
                )
                .as_bytes(),
            )],
            ..Spec::default()
        }),
        Opens,
    );

    // Decompression bombs and junk in every decoder -----------------------------
    add(
        "image-declared-huge",
        "an image that declares a million by a million pixels",
        image_page(stream(
            "/Type /XObject /Subtype /Image /Width 1000000 /Height 1000000 /BitsPerComponent 8 /ColorSpace /DeviceRGB",
            &junk(256, 19),
        )),
        OpensOrRefused,
    );
    add(
        "image-run-length-bomb",
        "a 100 KB image stream that expands to 400 MB, through two run-length filters",
        image_page(stream(
            "/Type /XObject /Subtype /Image /Width 20000 /Height 20000 /BitsPerComponent 8 /ColorSpace /DeviceGray /Filter [/RunLengthDecode /RunLengthDecode]",
            &double_run_length(50_000),
        )),
        OpensOrRefused,
    );
    add(
        "image-junk-flate",
        "a Flate image that is noise",
        junk_image("FlateDecode", "", 23),
        OpensOrRefused,
    );
    add(
        "image-junk-lzw",
        "an LZW image that is noise",
        junk_image("LZWDecode", "", 29),
        OpensOrRefused,
    );
    add(
        "image-junk-dct",
        "a JPEG image that is noise",
        junk_image("DCTDecode", "", 31),
        OpensOrRefused,
    );
    add(
        "image-junk-jpx",
        "a JPEG 2000 image that is noise",
        junk_image("JPXDecode", "", 37),
        OpensOrRefused,
    );
    add(
        "image-junk-jbig2",
        "a JBIG2 image that is noise",
        junk_image("JBIG2Decode", "", 41),
        OpensOrRefused,
    );
    add(
        "image-junk-ccitt",
        "a CCITT fax image with a million columns",
        junk_image(
            "CCITTFaxDecode",
            "/DecodeParms << /K -1 /Columns 1000000 /Rows 1000000 >>",
            43,
        ),
        OpensOrRefused,
    );
    for (name, what, key) in [
        (
            "font-junk-truetype",
            "an embedded TrueType font that is noise",
            "/FontFile2",
        ),
        (
            "font-junk-type1",
            "an embedded Type 1 font that is noise",
            "/FontFile",
        ),
        (
            "font-junk-cff",
            "an embedded CFF font that is noise",
            "/FontFile3",
        ),
    ] {
        add(
            name,
            what,
            doc(Spec {
                resources: "/Font << /F2 6 0 R >>".to_owned(),
                content: b"BT /F2 24 Tf 72 700 Td (Hostile) Tj ET".to_vec(),
                extra: vec![
                    b"<< /Type /Font /Subtype /TrueType /BaseFont /Evil /FirstChar 32 /LastChar 126 /FontDescriptor 7 0 R >>"
                        .to_vec(),
                    format!(
                        "<< /Type /FontDescriptor /FontName /Evil /Flags 32 /FontBBox [0 0 1000 1000] /ItalicAngle 0 /Ascent 800 /Descent -200 /CapHeight 700 /StemV 80 {key} 8 0 R >>"
                    )
                    .into_bytes(),
                    stream(
                        if key == "/FontFile3" { "/Subtype /Type1C" } else { "/Length1 4096" },
                        &junk(4096, 47),
                    ),
                ],
                ..Spec::default()
            }),
            OpensOrRefused,
        );
    }

    // Encryption ------------------------------------------------------------------
    add(
        "encrypted-needs-password",
        "a real encrypted file, no password given",
        encrypted_pdf("secret"),
        Refused(&[ErrorKind::PasswordRequired]),
    );
    add(
        "encrypted-unknown-handler",
        "an /Encrypt entry for a handler nobody has",
        doc(Spec {
            trailer: "/Encrypt << /Filter /Nope /V 99 /R 99 /Length 1000000 >>".to_owned(),
            ..Spec::default()
        }),
        OpensOrRefused,
    );
    add(
        "encrypted-key-length",
        "a standard handler with an absurd key length",
        doc(Spec {
            trailer: "/Encrypt << /Filter /Standard /V 2 /R 3 /Length 99999 /O (x) /U (y) /P -4 >> /ID [(a) (a)]"
                .to_owned(),
            ..Spec::default()
        }),
        OpensOrRefused,
    );

    samples
}

/// Two cross-reference sections whose `/Prev` entries point at each other.
fn xref_cycle() -> Vec<u8> {
    let objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>".to_vec(),
    ];
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }
    let section = |prev: usize| {
        let mut text = String::from("xref\n0 4\n0000000000 65535 f \n");
        for offset in &offsets {
            text.push_str(&format!("{offset:010} 00000 n \n"));
        }
        text.push_str(&format!(
            "trailer\n<< /Size 4 /Root 1 0 R /Prev {prev:010} >>\n"
        ));
        text
    };
    // Both sections are the same length, so each can name the other.
    let first = out.len();
    let second = first + section(0).len() + "startxref\n0000000000\n".len();
    out.extend_from_slice(section(second).as_bytes());
    out.extend_from_slice(format!("startxref\n{second:010}\n").as_bytes());
    out.extend_from_slice(section(first).as_bytes());
    out.extend_from_slice(format!("startxref\n{second:010}\n%%EOF\n").as_bytes());
    out
}
