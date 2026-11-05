//! PDFs for tests, built by hand so no binary fixture lives in the
//! repository. Behind the `test-support` feature.

/// A PDF of Letter pages, one per content stream, with Helvetica as `/F1`,
/// Helvetica-Bold as `/F2`, Times-Roman as `/F3` and Courier as `/F4` (the
/// standard fonts: proportional sans, proportional serif and fixed pitch).
pub fn pages_pdf(contents: &[String]) -> Vec<u8> {
    pages_pdf_on(contents, "[0 0 612 792]", 0)
}

/// As [`pages_pdf`], with the pages' `/MediaBox` (as PDF array text) and
/// `/Rotate` (degrees clockwise) given, for tests of rotated pages and of
/// pages whose box does not start at the origin.
pub fn pages_pdf_on(contents: &[String], media_box: &str, rotate: u16) -> Vec<u8> {
    let pages = contents.len();
    let mut objects: Vec<String> = Vec::new();
    // 1 catalog, 2 pages, 3 to 6 fonts, then a page and a content stream each.
    objects.push("<< /Type /Catalog /Pages 2 0 R >>".into());
    let kids: Vec<String> = (0..pages).map(|i| format!("{} 0 R", 7 + 2 * i)).collect();
    objects.push(format!(
        "<< /Type /Pages /Kids [{}] /Count {pages} >>",
        kids.join(" ")
    ));
    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".into());
    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>".into());
    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Times-Roman >>".into());
    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Courier >>".into());
    for (i, content) in contents.iter().enumerate() {
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox {media_box} /Rotate {rotate} /Resources << /Font << /F1 3 0 R /F2 4 0 R /F3 5 0 R /F4 6 0 R >> >> /Contents {} 0 R >>",
            8 + 2 * i
        ));
        objects.push(format!(
            "<< /Length {} >>\nstream\n{content}\nendstream",
            content.len()
        ));
    }
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
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
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

/// A PDF of `pages` Letter pages; each says "Kanso page N" in 36 pt Helvetica.
pub fn sample_pdf(pages: usize) -> Vec<u8> {
    let contents: Vec<String> = (0..pages)
        .map(|i| format!("BT /F1 36 Tf 72 700 Td (Kanso page {}) Tj ET", i + 1))
        .collect();
    pages_pdf(&contents)
}

/// Where the blue, orange and green bars of [`report_pdf`]'s first page are
/// (their centres, in points from the page's top-left) and their colours as
/// RGB, for tests that check a page's channels.
pub const REPORT_BARS: [([f32; 2], [u8; 3]); 3] = [
    ([102.0, 332.0], [51, 115, 217]),
    ([180.0, 302.0], [230, 89, 51]),
    ([258.0, 347.0], [64, 166, 102]),
];

/// A three-page report whose first page has a heading, text and coloured
/// bars, so a drawn page shows what the reader does with text and colour.
pub fn report_pdf() -> Vec<u8> {
    let first = [
        "0.2 0.45 0.85 rg 72 690 468 4 re f",
        "BT /F2 30 Tf 0 g 72 720 Td (Quarterly Report) Tj ET",
        "BT /F1 12 Tf 0.25 g 72 660 Td 17 TL",
        "(Revenue grew 12 percent over the last quarter, led by the new plans) Tj T*",
        "(and a steadier renewal rate. Costs held flat while the team grew by) Tj T*",
        "(three people, and the pilot in the north region finished early.) Tj ET",
        "BT /F2 14 Tf 0 g 72 580 Td (Revenue by region) Tj ET",
        "0.2 0.45 0.85 rg 72 400 60 120 re f",
        "0.9 0.35 0.2 rg 150 400 60 180 re f",
        "0.25 0.65 0.4 rg 228 400 60 90 re f",
        "BT /F1 10 Tf 0.4 g 72 380 Td (North) Tj 78 0 Td (South) Tj 78 0 Td (West) Tj ET",
    ]
    .join("\n");
    let rest = |n: usize| format!("BT /F1 24 Tf 72 700 Td (Page {n}) Tj ET");
    pages_pdf(&[first, rest(2), rest(3)])
}

/// A one-page PDF encrypted with the standard handler (RC4, 40-bit, revision
/// 2) under this user password, built by hand from the algorithms of the PDF
/// specification (7.6.3) so the password paths are tested for real.
pub fn encrypted_pdf(user_password: &str) -> Vec<u8> {
    use md5::{Digest, Md5};
    const PAD: [u8; 32] = [
        0x28, 0xBF, 0x4E, 0x5E, 0x4E, 0x75, 0x8A, 0x41, 0x64, 0x00, 0x4E, 0x56, 0xFF, 0xFA, 0x01,
        0x08, 0x2E, 0x2E, 0x00, 0xB6, 0xD0, 0x68, 0x3E, 0x80, 0x2F, 0x0C, 0xA9, 0xFE, 0x64, 0x53,
        0x69, 0x7A,
    ];
    fn rc4(key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut s: Vec<u8> = (0..=255).collect();
        let mut j = 0usize;
        for i in 0..256 {
            j = (j + s[i] as usize + key[i % key.len()] as usize) % 256;
            s.swap(i, j);
        }
        let (mut i, mut j) = (0usize, 0usize);
        data.iter()
            .map(|byte| {
                i = (i + 1) % 256;
                j = (j + s[i] as usize) % 256;
                s.swap(i, j);
                byte ^ s[(s[i] as usize + s[j] as usize) % 256]
            })
            .collect()
    }
    let owner = [0x5Au8; 32];
    let id = [0x11u8; 16];
    let permissions: i32 = -4;
    let mut padded = user_password.as_bytes().to_vec();
    padded.extend_from_slice(&PAD);
    padded.truncate(32);
    let mut hash = Md5::new();
    hash.update(&padded);
    hash.update(owner);
    hash.update(permissions.to_le_bytes());
    hash.update(id);
    let key = hash.finalize()[..5].to_vec();
    let user = rc4(&key, &PAD);
    let hex = |bytes: &[u8]| bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();

    // The content stream is object 5, generation 0.
    let mut object_key = key.clone();
    object_key.extend_from_slice(&[5, 0, 0, 0, 0]);
    let object_key = Md5::digest(&object_key)[..10].to_vec();
    let content = rc4(&object_key, b"BT /F1 36 Tf 72 700 Td (Locked) Tj ET");

    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    let mut object = |out: &mut Vec<u8>, number: usize, body: &[u8]| {
        offsets.push(out.len());
        out.extend_from_slice(format!("{number} 0 obj\n").as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    };
    object(&mut out, 1, b"<< /Type /Catalog /Pages 2 0 R >>");
    object(&mut out, 2, b"<< /Type /Pages /Kids [4 0 R] /Count 1 >>");
    object(
        &mut out,
        3,
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
    );
    object(
        &mut out,
        4,
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R >> >> /Contents 5 0 R >>",
    );
    let mut stream = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
    stream.extend_from_slice(&content);
    stream.extend_from_slice(b"\nendstream");
    object(&mut out, 5, &stream);
    object(
        &mut out,
        6,
        format!(
            "<< /Filter /Standard /V 1 /R 2 /O <{}> /U <{}> /P {permissions} >>",
            hex(&owner),
            hex(&user)
        )
        .as_bytes(),
    );
    let xref = out.len();
    out.extend_from_slice(b"xref\n0 7\n0000000000 65535 f \n");
    for offset in offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size 7 /Root 1 0 R /Encrypt 6 0 R /ID [<{0}> <{0}>] >>\nstartxref\n{xref}\n%%EOF\n",
            hex(&id)
        )
        .as_bytes(),
    );
    out
}

/// A one-page PDF whose body text is black and whose page carries a red
/// square annotation and a green text field, each with an appearance of its
/// own: a page drawn with annotations and form data on shows colour; the
/// worker draws neither.
pub fn annotated_pdf() -> Vec<u8> {
    let objects: [&str; 9] = [
        "<< /Type /Catalog /Pages 2 0 R /AcroForm << /Fields [8 0 R] /DA (/F1 0 Tf 0 g) /DR << /Font << /F1 3 0 R >> >> >> >>",
        "<< /Type /Pages /Kids [4 0 R] /Count 1 >>",
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R >> >> /Contents 5 0 R /Annots [6 0 R 8 0 R] >>",
        "<< /Length 40 >>\nstream\nBT /F1 24 Tf 72 700 Td (Annotated) Tj ET\nendstream",
        "<< /Type /Annot /Subtype /Square /Rect [100 300 300 500] /C [1 0 0] /IC [1 0 0] /F 4 /AP << /N 7 0 R >> >>",
        "<< /Type /XObject /Subtype /Form /BBox [0 0 200 200] /Length 26 >>\nstream\n1 0 0 rg 0 0 200 200 re f\nendstream",
        "<< /Type /Annot /Subtype /Widget /FT /Tx /T (name) /V (x) /Rect [350 300 550 400] /F 4 /P 4 0 R /DA (/F1 0 Tf 0 g) /AP << /N 9 0 R >> >>",
        "<< /Type /XObject /Subtype /Form /BBox [0 0 200 100] /Length 26 >>\nstream\n0 1 0 rg 0 0 200 100 re f\nendstream",
    ];
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(b"xref\n0 10\n0000000000 65535 f \n");
    for offset in offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!("trailer\n<< /Size 10 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n").as_bytes(),
    );
    out
}
