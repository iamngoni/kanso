//! What a facade call can fail with: one enum the shell maps once.
//!
//! The messages are sentences for a person, or short notes for a log. None
//! carries a note's text, a password or a key: the store's and the engine's
//! errors name what went wrong, never what was written.

use kanso_store::StoreError;

/// Why a call failed.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum KansoError {
    /// There is no such note, notebook or tag.
    #[error("not found")]
    NotFound,
    /// The note is locked and not open: its text can't be read or changed
    /// until the lock password is typed.
    #[error("the note is locked")]
    Locked,
    /// The note changed since the revision the caller saw. `current_rev` is
    /// its revision now; empty when the library can't say.
    #[error("the note changed meanwhile")]
    Conflict { current_rev: String },
    /// The request was wrong: a name too long, a password too short, a query
    /// that doesn't parse.
    #[error("{message}")]
    Invalid { message: String },
    /// The library on this phone couldn't be read or written.
    #[error("{message}")]
    Storage { message: String },
    /// The server couldn't be reached, or answered with a problem that a
    /// later try may clear.
    #[error("{message}")]
    Network { message: String },
    /// No account is signed in, or the server ended this device's session.
    #[error("not signed in")]
    SignedOut,
    /// The server or the phone's secret store refused: a wrong password, a
    /// turned-off account, a locked Keychain.
    #[error("{message}")]
    Auth { message: String },
    /// A bug: a call panicked. The panic was caught, and the library still
    /// works. See [`crate::boundary::guard`].
    #[error("internal error: {message}")]
    Internal { message: String },
}

impl KansoError {
    pub fn invalid(message: impl Into<String>) -> KansoError {
        KansoError::Invalid {
            message: message.into(),
        }
    }

    pub fn storage(message: impl Into<String>) -> KansoError {
        KansoError::Storage {
            message: message.into(),
        }
    }

    pub fn network(message: impl Into<String>) -> KansoError {
        KansoError::Network {
            message: message.into(),
        }
    }

    pub fn auth(message: impl Into<String>) -> KansoError {
        KansoError::Auth {
            message: message.into(),
        }
    }
}

impl From<StoreError> for KansoError {
    /// A store failure. A conflict says nothing of the current revision here;
    /// the calls that can say fill it in.
    fn from(err: StoreError) -> KansoError {
        match err {
            StoreError::NotFound => KansoError::NotFound,
            StoreError::Conflict => KansoError::Conflict {
                current_rev: String::new(),
            },
            StoreError::Invalid(message) => KansoError::Invalid { message },
            StoreError::GuardFailed => KansoError::storage(
                "The library is no longer bound to the account this started with.",
            ),
            StoreError::Sqlite(_)
            | StoreError::Json(_)
            | StoreError::Io(_)
            | StoreError::WrongKey
            | StoreError::NoEncryption => KansoError::storage(err.to_string()),
        }
    }
}

impl From<kanso_sync::SecretError> for KansoError {
    fn from(err: kanso_sync::SecretError) -> KansoError {
        KansoError::auth(format!("The phone's secret store refused: {err}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // no-prop-test: a table of the store's error kinds.
    #[test]
    fn store_errors_map_to_the_kinds_the_shell_tells_apart() {
        assert_eq!(KansoError::from(StoreError::NotFound), KansoError::NotFound);
        assert_eq!(
            KansoError::from(StoreError::Conflict),
            KansoError::Conflict {
                current_rev: String::new()
            }
        );
        assert_eq!(
            KansoError::from(StoreError::Invalid("too long".into())),
            KansoError::invalid("too long")
        );
        assert!(matches!(
            KansoError::from(StoreError::Io(std::io::Error::other("disk full"))),
            KansoError::Storage { .. }
        ));
        assert!(matches!(
            KansoError::from(StoreError::GuardFailed),
            KansoError::Storage { .. }
        ));
    }
}
