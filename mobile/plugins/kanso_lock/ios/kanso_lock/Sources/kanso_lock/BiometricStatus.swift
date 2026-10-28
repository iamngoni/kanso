import Foundation
import LocalAuthentication

/// The status codes of `kanso_biometric_answer` and the outcomes Dart is told.
enum BiometricStatus: Int32 {
  case ok = 0
  case cancelled = 1
  case invalidated = 2
  case unavailable = 3
  case failed = 4

  /// What Dart is told for an answer Rust took.
  var outcome: BiometricOutcome {
    switch self {
    case .ok: return .done
    case .cancelled: return .cancelled
    case .invalidated: return .invalidated
    case .unavailable: return .unavailable
    case .failed: return .failed
    }
  }

  /// The status of a failed `LAContext` evaluation.
  static func of(_ error: Error) -> BiometricStatus {
    guard let error = error as? LAError else { return .failed }
    switch error.code {
    case .userCancel, .appCancel, .systemCancel, .userFallback:
      return .cancelled
    case .biometryNotAvailable, .biometryNotEnrolled, .biometryLockout, .passcodeNotSet:
      return .unavailable
    default:
      return .failed
    }
  }

  /// The status of a failed Keychain read after a successful check: the item
  /// is gone (a biometrics change or a removed passcode deletes it) or no
  /// longer opens.
  static func ofKeychain(_ status: OSStatus) -> BiometricStatus {
    switch status {
    case errSecItemNotFound, errSecAuthFailed: return .invalidated
    case errSecUserCanceled: return .cancelled
    default: return .failed
    }
  }

  /// What Dart is told once Rust has replied to an answer for `self`.
  func outcome(after reply: RustAnswer.Reply) -> BiometricOutcome {
    switch reply {
    case .applied: return outcome
    case .wrongSecret: return .invalidated
    case .noTicket, .fault: return .failed
    }
  }
}
