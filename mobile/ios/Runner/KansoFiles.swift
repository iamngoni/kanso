// Kanso: KansoFiles.swift
// Licensed under AGPL-3.0-or-later (see LICENSE).

import Flutter
import UIKit
import UniformTypeIdentifiers

/// Files in and out of Kanso on iOS, behind the `app.kanso/files` channel.
///
/// Importing: the system's document picker chooses a file or a folder and
/// this copies the choice under `Caches/import/<name>`, which is where the
/// Dart side (`mobile/lib/import/`) and the Rust core read it, since the
/// picked place itself is not readable without a security scope. The copy is
/// removed with `clearPicked`.
///
/// Opening: Markdown and text files other apps open in Kanso (Info.plist's
/// `CFBundleDocumentTypes`, opened in place) reach `SceneDelegate`, which
/// calls `open(_:)`. Each is copied under `Caches/opened/<n>/<name>` with its
/// original address as its identity, queued, and announced to Dart with
/// `opened`. Dart takes the queue with `takeOpened` and removes the copies
/// with `discardOpened`; a document that arrives before Dart listens waits in
/// the queue. Nothing is decoded here.
final class KansoFiles: NSObject, UIDocumentPickerDelegate {
  static let shared = KansoFiles()

  private static let channelName = "app.kanso/files"

  private var channel: FlutterMethodChannel?
  private var pending: FlutterResult?
  private var queued: [[String: String]] = []
  private var taken: [URL] = []

  /// Answers the `app.kanso/files` channel.
  func register(with messenger: FlutterBinaryMessenger) {
    let channel = FlutterMethodChannel(
      name: KansoFiles.channelName,
      binaryMessenger: messenger
    )
    channel.setMethodCallHandler { [weak self] call, result in
      self?.handle(call, result: result)
    }
    self.channel = channel
    if !queued.isEmpty {
      channel.invokeMethod("opened", arguments: nil)
    }
  }

  // MARK: Channel

  private func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    switch call.method {
    case "pickFile":
      let arguments = call.arguments as? [String: Any]
      let extensions = arguments?["extensions"] as? [String] ?? []
      pick(folder: false, extensions: extensions, result: result)
    case "pickFolder":
      pick(folder: true, extensions: [], result: result)
    case "clearPicked":
      try? FileManager.default.removeItem(at: KansoFiles.folder("import"))
      result(nil)
    case "takeOpened":
      let batch: [String: Any] = [
        "staging": KansoFiles.folder("opened-staging").path,
        "files": queued,
      ]
      for entry in queued {
        if let path = entry["path"] {
          taken.append(URL(fileURLWithPath: path).deletingLastPathComponent())
        }
      }
      queued.removeAll()
      result(batch)
    case "discardOpened":
      for folder in taken {
        try? FileManager.default.removeItem(at: folder)
      }
      taken.removeAll()
      result(nil)
    default:
      result(FlutterMethodNotImplemented)
    }
  }

  private static func folder(_ name: String) -> URL {
    let caches = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask)[0]
    return caches.appendingPathComponent(name, isDirectory: true)
  }

  // MARK: Picking

  private func pick(folder: Bool, extensions: [String], result: @escaping FlutterResult) {
    guard pending == nil else {
      result(FlutterError(code: "busy", message: "A picker is already open.", details: nil))
      return
    }
    guard let presenter = KansoFiles.topViewController() else {
      result(FlutterError(code: "no_screen", message: "No screen to show the picker on.", details: nil))
      return
    }
    let picker: UIDocumentPickerViewController
    if folder {
      picker = UIDocumentPickerViewController(forOpeningContentTypes: [.folder])
    } else {
      // The system copies a picked file into the app's own temporary folder, so
      // no security scope is needed to read it.
      picker = UIDocumentPickerViewController(
        forOpeningContentTypes: KansoFiles.types(for: extensions),
        asCopy: true
      )
    }
    picker.allowsMultipleSelection = false
    picker.delegate = self
    pending = result
    presenter.present(picker, animated: true)
  }

  /// The types for file `extensions`; any file when the list is empty or one
  /// of them has no type the system knows.
  private static func types(for extensions: [String]) -> [UTType] {
    let types = extensions.compactMap { UTType(filenameExtension: $0) }
    return types.isEmpty || types.count != extensions.count ? [.item] : types
  }

  func documentPicker(_ controller: UIDocumentPickerViewController, didPickDocumentsAt urls: [URL]) {
    guard let result = pending, let url = urls.first else {
      pending?(nil)
      pending = nil
      return
    }
    pending = nil
    DispatchQueue.global(qos: .userInitiated).async {
      do {
        let copy = try KansoFiles.copy(url, toFolder: KansoFiles.folder("import"), replacing: true)
        DispatchQueue.main.async {
          result(["path": copy.path, "name": copy.lastPathComponent])
        }
      } catch {
        DispatchQueue.main.async {
          result(FlutterError(code: "copy_failed", message: error.localizedDescription, details: nil))
        }
      }
    }
  }

  func documentPickerWasCancelled(_ controller: UIDocumentPickerViewController) {
    pending?(nil)
    pending = nil
  }

  /// Copies the file or folder at `url` into `folder` under its own name; the
  /// copy. The place is read under its security scope, if it has one.
  private static func copy(_ url: URL, toFolder folder: URL, replacing: Bool) throws -> URL {
    let scoped = url.startAccessingSecurityScopedResource()
    defer {
      if scoped {
        url.stopAccessingSecurityScopedResource()
      }
    }
    let files = FileManager.default
    if replacing {
      try? files.removeItem(at: folder)
    }
    try files.createDirectory(at: folder, withIntermediateDirectories: true)
    let target = folder.appendingPathComponent(url.lastPathComponent)
    try? files.removeItem(at: target)
    try files.copyItem(at: url, to: target)
    return target
  }

  // MARK: Opening

  /// Queues the documents the system opened in the app. Other addresses
  /// (Kanso's own links) are not documents and are left to the router.
  func open(_ urls: [URL]) {
    var added = false
    for url in urls where url.isFileURL {
      let folder = KansoFiles.folder("opened").appendingPathComponent(UUID().uuidString, isDirectory: true)
      let name = url.lastPathComponent
      if let copy = try? KansoFiles.copy(url, toFolder: folder, replacing: false) {
        queued.append([
          "path": copy.path,
          "name": copy.lastPathComponent,
          "identity": url.absoluteString,
        ])
      } else {
        queued.append([
          "path": folder.appendingPathComponent(name).path,
          "name": name,
          "identity": url.absoluteString,
        ])
      }
      added = true
    }
    if added {
      channel?.invokeMethod("opened", arguments: nil)
    }
  }

  private static func topViewController() -> UIViewController? {
    let scenes = UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }
    let window = scenes.flatMap { $0.windows }.first { $0.isKeyWindow }
    var top = window?.rootViewController
    while let presented = top?.presentedViewController {
      top = presented
    }
    return top
  }
}
