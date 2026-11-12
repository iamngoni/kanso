//! What Kanso costs, as data (issue #390, slice 43). The words and the sums
//! are in [`super::cost`]; the desktop and the phone both show a
//! [`CostSummary`], so they say the same thing.

pub const KIB: u64 = 1024;
pub const MIB: u64 = 1024 * KIB;
pub const GIB: u64 = 1024 * MIB;

/// The default notes space of an account (`quota_doc_bytes`,
/// cloud/migrations/0002_accounts.sql). The Price page's `quota.ts` says the
/// same number; the one source in Rust is here.
pub const NOTES_BYTES: u64 = 100 * MIB;
/// The default files space (`quota_file_bytes`, raised by
/// cloud/migrations/0017_pdf_file_size.sql).
pub const FILES_BYTES: u64 = GIB;
/// The default upload cap per UTC day (`upload_bytes_per_day`).
pub const UPLOAD_PER_DAY_BYTES: u64 = GIB;

/// The Price page, which the row's "Learn more" opens.
pub const PRICE_URL: &str = "https://kansoapp.app/price";

/// The row Settings › Account shows on both apps.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CostSummary {
    /// "Free. No subscription."
    pub headline: String,
    /// "Sync space: 23 MiB of 100 MiB notes · 410 MiB of 1 GiB files."
    pub space_line: String,
    /// Where "Learn more" goes.
    pub learn_more_url: String,
}
