//! What the ledger keeps (docs/METRICS.md › Sending): the data only. The
//! rules that read and change it are in `ledger.rs`.

use serde::Serialize;

use crate::ranges::SessionLength;
use crate::values::{Day, Id};

/// One feature used `n` times on `day`, not yet reported. `feature` is the
/// ledger key (`import:<source>` for an import).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Counted {
    pub day: Day,
    pub feature: String,
    pub n: u32,
}

/// A session that ended on `day` and is not yet reported.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Ended {
    pub day: Day,
    pub length: SessionLength,
}

/// How many `app_opened` were reported for an ID on the ledger's day.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Opened {
    pub id: Id,
    pub n: u8,
}

/// How many sessions were counted on `day`, so the day's cap holds even
/// after the batch that carried them is settled and removed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SessionsCounted {
    pub day: Day,
    pub n: u8,
}

/// The on-device ledger: what has been reported and what waits, so a
/// restart never counts anything twice. A day earlier than the one a list
/// holds is dropped, never sent late.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ledger {
    /// This install's `i_` ID, made the first time it is needed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) install_id: Option<Id>,
    /// The UTC day `active_sent` and `opened_sent` are for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) day: Option<Day>,
    /// The IDs whose `app_active` that day is done.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) active_sent: Vec<Id>,
    /// The `app_opened` already reported that day, per ID.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) opened_sent: Vec<Opened>,
    /// This launch's `app_opened` is not yet reported.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub(crate) launch_pending: bool,
    /// Features used since the last batch.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) counts: Vec<Counted>,
    /// Sessions that ended since the last batch.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) sessions: Vec<Ended>,
    /// Sessions counted on the latest day one ended, for the daily cap.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) sessions_counted: Option<SessionsCounted>,
    /// Sync errors since the last `app_active` was reported.
    #[serde(skip_serializing_if = "is_zero")]
    pub(crate) sync_errors: u32,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}
