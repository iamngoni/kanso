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

  /// The App Group every Kanso target on this phone belongs to; the library
  /// lives in its container (`lib/core/configs/core_module.dart`).
  static let appGroup = "group.app.kanso.Kanso"

  /// The library's database file: the one the app opens. That is
  /// `<App Group container>/Library/Application Support/kanso/library.db`,
  /// or the app's own `<Application Support>/kanso/library.db` for a build
  /// without the group. Nil when neither exists yet: a background job never
  /// makes a library of its own, which would sync the whole account into a
  /// stray database the app never opens.
  public static func libraryPath() -> String? {
    let files = FileManager.default
    var candidates: [URL] = []
    if let container = files.containerURL(forSecurityApplicationGroupIdentifier: appGroup) {
      candidates.append(
        container.appendingPathComponent("Library", isDirectory: true)
          .appendingPathComponent("Application Support", isDirectory: true)
          .appendingPathComponent("kanso", isDirectory: true)
          .appendingPathComponent("library.db"))
    }
    if let support = files.urls(for: .applicationSupportDirectory, in: .userDomainMask).first {
      candidates.append(
        support.appendingPathComponent("kanso", isDirectory: true)
          .appendingPathComponent("library.db"))
    }
    return candidates.first { files.fileExists(atPath: $0.path) }?.path
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
    let code = path.withCString { entry($0, millis) }
    // A sync that ran with the app suspended or about to be must not leave
    // SQLite locks on the App Group's files behind (0xdead10cc).
    releaseLibrary()
    return code
  }

  /// `int32_t kanso_library_release(const char *library_path)`
  private typealias Release = @convention(c) (UnsafePointer<CChar>?) -> Int32

  /// Closes the library's database connections in this process, so none holds
  /// a lock on the library while the app is suspended. The next use opens
  /// them again. Blocks while a write is in progress; call it off the main
  /// thread.
  public static func releaseLibrary() {
    guard let path = libraryPath(), let symbol = symbol("kanso_library_release") else { return }
    let release = unsafeBitCast(symbol, to: Release.self)
    _ = path.withCString { release($0) }
  }

  /// The Rust library is a dynamic framework that Dart loads, so the app does
  /// not link it: look the entry up in the process, and failing that load the
  /// framework from the app bundle (a launch for a background task has run no
  /// Dart yet).
  private static func entry() -> Entry? {
    guard let symbol = symbol("kanso_background_sync") else { return nil }
    return unsafeBitCast(symbol, to: Entry.self)
  }

  private static func symbol(_ name: String) -> UnsafeMutableRawPointer? {
    var symbol = dlsym(UnsafeMutableRawPointer(bitPattern: -2), name)  // RTLD_DEFAULT
    if symbol == nil, let frameworks = Bundle.main.privateFrameworksPath {
      let image = "\(frameworks)/kanso_mobile_bridge.framework/kanso_mobile_bridge"
      if let handle = dlopen(image, RTLD_NOW) {
        symbol = dlsym(handle, name)
      }
    }
    return symbol
  }
}
