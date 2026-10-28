import Darwin
import Foundation

/// Hands a biometric check's answer to Rust: `kanso_biometric_answer`
/// (`mobile/rust/src/biometric.rs`).
///
/// The answer holds the biometric secret, so it goes from here to Rust and is
/// never returned to Dart, logged or kept.
enum RustAnswer {
  /// `int32_t kanso_biometric_answer(uint64_t ticket, int32_t status,
  /// const uint8_t *secret, size_t len)`
  private typealias Entry =
    @convention(c) (UInt64, Int32, UnsafePointer<UInt8>?, Int) -> Int32

  /// What Rust says about an answer.
  enum Reply: Equatable {
    /// Rust took it.
    case applied
    /// No ticket waits (it expired, or was answered).
    case noTicket
    /// The secret did not open the kept keys: they are gone.
    case wrongSecret
    /// A fault in Rust, or the library is not there.
    case fault
  }

  /// Gives `status` (and the secret for status 0) to the check waiting on
  /// `ticket`. The caller's bytes are not copied; they stay the caller's to
  /// zero.
  static func deliver(ticket: UInt64, status: Int32, secret: [UInt8]?) -> Reply {
    guard let entry = entry() else { return .fault }
    let code: Int32
    if let secret {
      code = secret.withUnsafeBufferPointer { entry(ticket, status, $0.baseAddress, $0.count) }
    } else {
      code = entry(ticket, status, nil, 0)
    }
    return reply(code)
  }

  /// The meaning of the entry's return value.
  static func reply(_ code: Int32) -> Reply {
    switch code {
    case 1: return .applied
    case 0: return .noTicket
    case -1: return .wrongSecret
    default: return .fault
    }
  }

  /// The Rust library is a dynamic framework that Dart loads, so the app does
  /// not link it: look the entry up in the process, and failing that load the
  /// framework from the app bundle.
  private static func entry() -> Entry? {
    let name = "kanso_biometric_answer"
    var symbol = dlsym(UnsafeMutableRawPointer(bitPattern: -2), name)  // RTLD_DEFAULT
    if symbol == nil, let frameworks = Bundle.main.privateFrameworksPath {
      let image = "\(frameworks)/kanso_mobile_bridge.framework/kanso_mobile_bridge"
      if let handle = dlopen(image, RTLD_NOW) {
        symbol = dlsym(handle, name)
      }
    }
    guard let symbol else { return nil }
    return unsafeBitCast(symbol, to: Entry.self)
  }
}
