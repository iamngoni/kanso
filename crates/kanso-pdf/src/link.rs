//! Links that open a PDF: `inkdrop://file:<id>#page=4&h=h7k2m9`.

use kanso_core::links::FILE_LINK_PREFIX;

/// Where a PDF link points.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PdfLink {
    /// The attachment id without its `file:` prefix.
    pub file_id: String,
    /// The 1-based page, when the link names one.
    pub page: Option<u32>,
    /// A highlight id, a hint only.
    pub highlight: Option<String>,
}

impl PdfLink {
    /// Parses an attachment link with an optional fragment. `None` when it is
    /// not an attachment link. A bad `page` is ignored, not an error.
    pub fn parse(url: &str) -> Option<PdfLink> {
        let rest = url.strip_prefix(FILE_LINK_PREFIX)?;
        let (id, fragment) = match rest.split_once('#') {
            Some((id, f)) => (id, f),
            None => (rest, ""),
        };
        if id.is_empty() || id.contains(['/', '?', ' ']) {
            return None;
        }
        let mut link = PdfLink {
            file_id: id.to_string(),
            page: None,
            highlight: None,
        };
        for pair in fragment.split('&') {
            match pair.split_once('=') {
                Some(("page", v)) => {
                    link.page = v.parse::<u32>().ok().filter(|p| *p >= 1);
                }
                Some(("h", v))
                    if !v.is_empty()
                        && v.len() <= 16
                        && v.bytes().all(|b| b.is_ascii_alphanumeric()) =>
                {
                    link.highlight = Some(v.to_string());
                }
                _ => {}
            }
        }
        Some(link)
    }

    /// The link as written into a note.
    pub fn to_url(&self) -> String {
        let mut url = format!("{FILE_LINK_PREFIX}{}", self.file_id);
        let mut parts = Vec::new();
        if let Some(p) = self.page {
            parts.push(format!("page={p}"));
        }
        if let Some(h) = &self.highlight {
            parts.push(format!("h={h}"));
        }
        if !parts.is_empty() {
            url.push('#');
            url.push_str(&parts.join("&"));
        }
        url
    }

    /// The page to open in a document of `pages` pages: the link's page
    /// clamped into range, else the first.
    pub fn target_page(&self, pages: u32) -> u32 {
        self.page.unwrap_or(1).clamp(1, pages.max(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn parses_the_spec_link() {
        let l = PdfLink::parse("inkdrop://file:abc123#page=4&h=h7k2m9").unwrap();
        assert_eq!(l.file_id, "abc123");
        assert_eq!(l.page, Some(4));
        assert_eq!(l.highlight.as_deref(), Some("h7k2m9"));
        assert_eq!(l.to_url(), "inkdrop://file:abc123#page=4&h=h7k2m9");
    }

    #[test]
    fn plain_and_odd_links() {
        assert_eq!(PdfLink::parse("inkdrop://file:a").unwrap().page, None);
        assert_eq!(
            PdfLink::parse("inkdrop://file:a#page=0").unwrap().page,
            None
        );
        assert_eq!(
            PdfLink::parse("inkdrop://file:a#page=x&h=")
                .unwrap()
                .highlight,
            None
        );
        assert!(PdfLink::parse("inkdrop://note/a").is_none());
        assert!(PdfLink::parse("inkdrop://file:").is_none());
        assert!(PdfLink::parse("https://example.com").is_none());
    }

    #[test]
    fn the_page_is_clamped_to_the_document() {
        let l = PdfLink::parse("inkdrop://file:a#page=99").unwrap();
        assert_eq!(l.target_page(12), 12);
        assert_eq!(
            PdfLink::parse("inkdrop://file:a").unwrap().target_page(12),
            1
        );
        assert_eq!(l.target_page(0), 1);
    }

    proptest! {
        #[test]
        fn links_round_trip(id in "[a-zA-Z0-9]{1,20}", page in proptest::option::of(1u32..100_000), h in proptest::option::of("[a-z0-9]{1,16}")) {
            let l = PdfLink { file_id: id, page, highlight: h };
            prop_assert_eq!(PdfLink::parse(&l.to_url()), Some(l));
        }

        #[test]
        fn parse_never_panics(s in "\\PC{0,80}") {
            let _ = PdfLink::parse(&s);
            let _ = PdfLink::parse(&format!("inkdrop://file:{s}"));
        }

        #[test]
        fn target_page_is_always_in_range(page in proptest::option::of(0u32..u32::MAX), pages in 0u32..5000) {
            let l = PdfLink { file_id: "a".into(), page, highlight: None };
            let t = l.target_page(pages);
            prop_assert!(t >= 1 && t <= pages.max(1));
        }
    }
}
