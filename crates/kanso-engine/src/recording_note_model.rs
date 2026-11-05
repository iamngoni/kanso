//! The data a recording's note is made of.

/// One piece of a transcript: what was said from `start_secs` on.
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    /// Seconds from the start of the recording.
    pub start_secs: u32,
    /// The words, already tidied.
    pub text: String,
    /// Who said them (`You` for the microphone, `Meeting` for system
    /// audio), when the recording kept the two apart.
    pub speaker: Option<&'static str>,
}

/// Where a `kanso-audio:` link points.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioMoment {
    /// The attachment's file ID, with its `file:` prefix.
    pub file_id: String,
    /// Seconds into the recording.
    pub seconds: u32,
}
