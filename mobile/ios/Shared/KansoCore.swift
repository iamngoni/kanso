// Kanso — KansoCore.swift
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import Foundation

/// What saving the inbox's drops did, as the library reported it.
struct CaptureResult: Decodable, Equatable {
  /// The IDs of the notes that were made.
  var notes: [String]
  /// Sentences about what was left out or refused, written for the person.
  var problems: [String]
  /// How many drops were kept to be tried again.
  var kept: Int
}

/// Calls into the Rust library from places that have no Flutter engine: the
/// share extension and App Intents.
///
/// The library is `kanso_mobile_bridge.framework` in the app's `Frameworks`
/// folder. The app finds it in its own bundle; the extension, which lives in
/// `Kanso.app/PlugIns/<name>.appex`, finds it two folders up. The two C
/// functions are described in `mobile/docs/capture.md`.
enum KansoCore {
  private typealias CaptureFunction = @convention(c) (
    UnsafePointer<CChar>?, UnsafePointer<CChar>?
  ) -> UnsafeMutablePointer<CChar>?
  private typealias FreeFunction = @convention(c) (UnsafeMutablePointer<CChar>?) -> Void

  /// The framework's folder and executable name.
  private static let frameworkName = "kanso_mobile_bridge"

  /// Where the framework's binary is, from this process's bundle.
  static func frameworkBinaryURLs() -> [URL] {
    var frameworks: [URL] = []
    if let own = Bundle.main.privateFrameworksURL { frameworks.append(own) }
    // `Kanso.app/PlugIns/X.appex` → `Kanso.app/Frameworks`.
    frameworks.append(
      Bundle.main.bundleURL
        .appendingPathComponent("../../Frameworks", isDirectory: true)
        .standardized)
    return frameworks.map {
      $0.appendingPathComponent("\(frameworkName).framework/\(frameworkName)")
    }
  }

  /// Saves every finished drop in the location's inbox into its library.
  ///
  /// Returns `nil` when the drops stay where they are for the app to save:
  /// the app hasn't made the library here yet (the library's folder is moved
  /// into the App Group by the app's first launch with this build, and the
  /// library must not be created by anything else), or the Rust library
  /// couldn't be loaded.
  ///
  /// Blocks until the library is done; don't call it on the main thread.
  static func captureInbox(in location: KansoLibraryLocation) -> CaptureResult? {
    guard location.databaseExists else { return nil }
    guard let (capture, free) = loadFunctions() else { return nil }

    let pointer = capture(location.database.path, location.inbox.path)
    guard let pointer else { return nil }
    defer { free(pointer) }
    return try? JSONDecoder().decode(
      CaptureResult.self, from: Data(bytes: pointer, count: strlen(pointer)))
  }

  /// The two functions, from the process itself when the library is loaded
  /// already (the app), otherwise from the framework file.
  private static func loadFunctions() -> (CaptureFunction, FreeFunction)? {
    func symbols(in handle: UnsafeMutableRawPointer?) -> (CaptureFunction, FreeFunction)? {
      guard
        let capture = dlsym(handle, "kanso_capture_inbox"),
        let free = dlsym(handle, "kanso_string_free")
      else { return nil }
      return (
        unsafeBitCast(capture, to: CaptureFunction.self),
        unsafeBitCast(free, to: FreeFunction.self)
      )
    }

    if let found = symbols(in: UnsafeMutableRawPointer(bitPattern: -2)) {  // RTLD_DEFAULT
      return found
    }
    for url in frameworkBinaryURLs() where FileManager.default.fileExists(atPath: url.path) {
      if let handle = dlopen(url.path, RTLD_NOW | RTLD_LOCAL), let found = symbols(in: handle) {
        return found
      }
    }
    return nil
  }

  /// A sentence for the person about what saving did, in Kanso's plain words.
  static func summary(of result: CaptureResult?) -> String {
    let keptForApp = "Saved. Kanso will add it to your Inbox the next time you open it."
    guard let result else { return keptForApp }
    let problems = result.problems.joined(separator: " ")
    if !result.notes.isEmpty {
      return problems.isEmpty ? "Saved to your Inbox." : "Saved to your Inbox. \(problems)"
    }
    if !problems.isEmpty { return problems }
    return result.kept > 0 ? keptForApp : "There was nothing to save."
  }
}
