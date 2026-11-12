//! The words of Settings › Account › "What Kanso costs" (issue #390, slice
//! 43): Kanso is free, and how much sync space the account has used. The
//! limits are the account's own when the server reported them, otherwise the
//! defaults in [`super::cost_model`].

use kanso_sync::Usage;

use super::cost_model::{
    CostSummary, FILES_BYTES, GIB, MIB, NOTES_BYTES, PRICE_URL, UPLOAD_PER_DAY_BYTES,
};

/// The row's title.
pub const TITLE: &str = "What Kanso costs";
/// The row's answer.
pub const HEADLINE: &str = "Free. No subscription.";
/// The link's words.
pub const LEARN_MORE: &str = "Learn more";

/// A size in binary units, as the Price page prints it: "23 MiB", "1 GiB",
/// "1.5 GiB". Under a MiB it says so rather than print KiB.
pub fn format_size(bytes: u64) -> String {
    if bytes == 0 {
        "0 MiB".to_owned()
    } else if bytes < MIB {
        "under 1 MiB".to_owned()
    } else if bytes < GIB {
        format!("{} MiB", bytes / MIB)
    } else {
        let tenths = (u128::from(bytes) * 10 + u128::from(GIB) / 2) / u128::from(GIB);
        match tenths % 10 {
            0 => format!("{} GiB", tenths / 10),
            part => format!("{}.{part} GiB", tenths / 10),
        }
    }
}

/// The row for an account: its use of the space when the server has said,
/// otherwise the limits every account starts with.
pub fn cost_summary(usage: Option<&Usage>) -> CostSummary {
    let space_line = match usage {
        Some(usage) => format!(
            "Sync space: {} of {} notes · {} of {} files.",
            format_size(usage.doc_bytes),
            format_size(usage.quota.doc_bytes),
            format_size(usage.file_bytes),
            format_size(usage.quota.file_bytes),
        ),
        None => format!(
            "Sync space: {} of notes · {} of files · {} of uploads a day.",
            format_size(NOTES_BYTES),
            format_size(FILES_BYTES),
            format_size(UPLOAD_PER_DAY_BYTES),
        ),
    };
    CostSummary {
        headline: HEADLINE.to_owned(),
        space_line,
        learn_more_url: PRICE_URL.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_sync::Quota;
    use proptest::prelude::*;

    fn usage(doc_bytes: u64, file_bytes: u64) -> Usage {
        Usage {
            docs: 850,
            doc_bytes,
            files: 15,
            file_bytes,
            writes_today: 0,
            quota: Quota {
                docs: 50_000,
                doc_bytes: NOTES_BYTES,
                files: 10_000,
                file_bytes: FILES_BYTES,
                writes_per_day: 20_000,
            },
        }
    }

    #[test]
    fn the_issue_example_reads_as_written() {
        let summary = cost_summary(Some(&usage(23 * MIB, 410 * MIB)));
        assert_eq!(summary.headline, "Free. No subscription.");
        assert_eq!(
            summary.space_line,
            "Sync space: 23 MiB of 100 MiB notes · 410 MiB of 1 GiB files."
        );
        assert_eq!(summary.learn_more_url, "https://kansoapp.app/price");
    }

    #[test]
    fn without_usage_it_gives_the_default_limits() {
        assert_eq!(
            cost_summary(None).space_line,
            "Sync space: 100 MiB of notes · 1 GiB of files · 1 GiB of uploads a day."
        );
    }

    #[test]
    fn sizes_at_the_edges() {
        assert_eq!(format_size(0), "0 MiB");
        assert_eq!(format_size(1), "under 1 MiB");
        assert_eq!(format_size(MIB - 1), "under 1 MiB");
        assert_eq!(format_size(MIB), "1 MiB");
        assert_eq!(format_size(GIB - 1), "1023 MiB");
        assert_eq!(format_size(GIB), "1 GiB");
        assert_eq!(format_size(GIB + GIB / 2), "1.5 GiB");
        assert!(format_size(u64::MAX).ends_with(" GiB"));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        // Whole numbers of MiB under a GiB, and of GiB, print as that
        // number, so the limits read the way the Price page says them.
        #[test]
        fn whole_units_print_whole(mib in 1u64..1024, gib in 1u64..4096) {
            prop_assert_eq!(format_size(mib * MIB), format!("{mib} MiB"));
            prop_assert_eq!(format_size(gib * GIB), format!("{gib} GiB"));
        }

        // Any size has a unit, and a GiB or more never drops to MiB.
        #[test]
        fn every_size_has_a_unit(bytes in any::<u64>()) {
            let said = format_size(bytes);
            prop_assert!(said.ends_with("MiB") || said.ends_with("GiB"), "{}", said);
            if bytes >= GIB {
                prop_assert!(said.ends_with("GiB"));
            }
        }

        // The row always carries the headline and the link, and names both
        // spaces, for any usage the server could report.
        #[test]
        fn the_row_is_always_complete(doc in any::<u64>(), files in any::<u64>()) {
            let summary = cost_summary(Some(&usage(doc, files)));
            prop_assert_eq!(&summary.headline, HEADLINE);
            prop_assert_eq!(&summary.learn_more_url, PRICE_URL);
            prop_assert!(summary.space_line.starts_with("Sync space: "));
            prop_assert!(summary.space_line.contains(" notes · "));
            prop_assert!(summary.space_line.ends_with(" files."));
        }
    }
}
