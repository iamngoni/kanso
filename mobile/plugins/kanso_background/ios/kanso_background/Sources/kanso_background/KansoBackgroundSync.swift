import Darwin
import Foundation

/// The sync that runs with no app: calls the Rust library's
/// `kanso_background_sync` (`mobile/rust/src/background.rs`).
///
/// The `BGAppRefreshTask` handler uses it, and so does the silent-push handler
/// (#48 step 8b).
public enum KansoBackgroundSync {
  /// The budget a background refresh gets, in seconds.
  public static let refreshBudget: TimeInterval = 25

  /// `int32_t kanso_background_sync(const char *library_path, uint32_t budget_ms)`
  private typealias Entry = @convention(c) (UnsafePointer<CChar>?, UInt32) -> Int32

  /// The result for anything outside the sync itself, as the Rust side
  /// numbers it: the library or the entry is missing.
  public static let internalError: Int32 = -5

  /// The library's database file: the one the app opens
  /// (`<Application Support>/kanso/library.db`).
  public static func libraryPath() -> String? {
    guard
      let support = FileManager.default.urls(
        for: .applicationSupportDirectory, in: .userDomainMask
      ).first
    else { return nil }
    return support.appendingPathComponent("kanso", isDirectory: true)
      .appendingPathComponent("library.db").path
  }

  /// Syncs the library once, within `budget` seconds, off the calling thread.
  /// The result is the code of `background.rs`: 0 or 1 done, -1 signed out,
  /// -2 the Keychain refused, -3 the server, -4 out of time, -5 internal.
  public static func run(budget: TimeInterval) async -> Int32 {
    await withCheckedContinuation { continuation in
      DispatchQueue.global(qos: .utility).async {
        continuation.resume(returning: runBlocking(budget: budget))
      }
    }
  }

  /// [run], on the calling thread.
  static func runBlocking(budget: TimeInterval) -> Int32 {
    guard let path = libraryPath(), let entry = entry() else {
      return internalError
    }
    let millis = UInt32(clamping: Int((max(budget, 0) * 1000).rounded()))
    return path.withCString { entry($0, millis) }
  }

  /// The Rust library is a dynamic framework that Dart loads, so the app does
  /// not link it: look the entry up in the process, and failing that load the
  /// framework from the app bundle (a launch for a background task has run no
  /// Dart yet).
  private static func entry() -> Entry? {
    let name = "kanso_background_sync"
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
