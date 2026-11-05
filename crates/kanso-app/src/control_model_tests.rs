use super::*;
use proptest::prelude::*;
use proptest::test_runner::RngSeed;

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 256,
        rng_seed: RngSeed::Fixed(0x0063_7472_6c00),
        ..ProptestConfig::default()
    }
}

proptest! {
    #![proptest_config(config())]

    /// A command goes down the socket and comes back the same, and the
    /// line carries nothing but what was asked.
    #[test]
    fn prop_a_command_survives_the_socket(
        name in "[ -~]{1,20}",
        access in prop::sample::select(KeyAccess::ALL.to_vec()),
        scopes in prop::collection::vec("[ -~]{1,10}", 0..4),
        target in "[ -~]{1,20}",
        which in 0u8..3,
    ) {
        let command = match which {
            0 => Command::Create { name, access, scopes },
            1 => Command::List,
            _ => Command::Revoke { target },
        };
        let request = Request::of(&command).unwrap();
        let line = serde_json::to_string(&request).unwrap();
        prop_assert!(!line.contains('\n'));
        let back: Request = serde_json::from_str(&line).unwrap();
        prop_assert_eq!(back.command(), Ok(command));
    }

    /// Anything that is not a request is refused, never a panic.
    #[test]
    fn prop_other_lines_are_not_requests(line in "[ -~]{0,80}") {
        if let Ok(request) = serde_json::from_str::<Request>(&line) {
            let _ = request.command();
        }
    }
}

// no-prop-test: help has no request by definition.
#[test]
fn help_needs_no_process_and_a_bad_access_is_refused() {
    assert_eq!(Request::of(&Command::Help), None);
    let bad = Request::Create {
        name: "x".into(),
        access: "admin".into(),
        scopes: vec![],
    };
    assert!(
        bad.command()
            .unwrap_err()
            .contains("read, suggest or write")
    );
}
