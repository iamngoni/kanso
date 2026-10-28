import Darwin
import Foundation

/// Hands a ceremony's answer to Rust: `kanso_passkey_answer`
/// (`mobile/rust/src/passkey.rs`).
///
/// The answer holds the PRF output, so it goes from here to Rust and is never
/// returned to Dart, logged or kept.
enum RustAnswer {
  /// `int32_t kanso_passkey_answer(uint64_t ticket, const char *json)`
  private typealias Entry = @convention(c) (UInt64, UnsafePointer<CChar>?) -> Int32

  /// Gives `json` to the ceremony waiting on `ticket`. True when one took it.
  static func deliver(ticket: UInt64, json: String) -> Bool {
    guard let entry = entry() else { return false }
    return json.withCString { entry(ticket, $0) } == 1
  }

  /// The Rust library is a dynamic framework that Dart loads, so the app does
  /// not link it: look the entry up in the process, and failing that load the
  /// framework from the app bundle.
  private static func entry() -> Entry? {
    let name = "kanso_passkey_answer"
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
