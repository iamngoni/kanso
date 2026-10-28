import Flutter
import UIKit

/// A note in a window of its own on iPadOS: a scene per note, each with a
/// Flutter engine of its own in this process (the Dart side opens the one
/// Rust library from all of them, so typing and the lock session are shared).
///
/// The Dart side talks to this over the `app.kanso/windows` channel, which
/// every engine has: the main engine's answers for the main window, and each
/// note window's for its own scene. See `lib/window/` in the Flutter app.
final class NoteWindows {
  static let shared = NoteWindows()

  /// The activity type of a note's scene, and what its user info holds.
  static let activityType = "app.kanso.Kanso.note"
  static let configurationName = "NoteWindow"
  private static let noteIdKey = "noteId"
  private static let environmentKey = "environment"
  private static let libraryFolderKey = "libraryFolder"

  private static let channelName = "app.kanso/windows"
  private static let entryPoint = "noteWindowMain"
  private static let entryLibrary = "package:kanso/note_window_main.dart"

  /// What one note's scene is.
  final class Window {
    let noteId: String
    let environment: String
    let libraryFolder: String
    weak var session: UISceneSession?
    weak var scene: UIScene?
    var engine: FlutterEngine?
    var channel: FlutterMethodChannel?

    init(noteId: String, environment: String, libraryFolder: String) {
      self.noteId = noteId
      self.environment = environment
      self.libraryFolder = libraryFolder
    }

    /// What the scene is restored from: the note, and where its library is.
    /// Never the note's title, which a locked note keeps out of the system.
    var activity: NSUserActivity {
      let activity = NSUserActivity(activityType: NoteWindows.activityType)
      activity.userInfo = [
        NoteWindows.noteIdKey: noteId,
        NoteWindows.environmentKey: environment,
        NoteWindows.libraryFolderKey: libraryFolder,
      ]
      return activity
    }

    var entrypointArgs: [String] { [noteId, environment, libraryFolder] }
  }

  private lazy var group = FlutterEngineGroup(name: "app.kanso.windows", project: nil)
  private var mainChannel: FlutterMethodChannel?
  private(set) weak var mainSession: UISceneSession?

  /// The scenes of the notes that have one, by note id.
  private var windows: [String: Window] = [:]
  /// Notes whose scene was asked for and has not connected yet, so that a
  /// second request does not make a second scene.
  private var pending: Set<String> = []

  // MARK: The channel

  /// Answers the channel for the main engine.
  func registerMain(messenger: FlutterBinaryMessenger) {
    mainChannel = makeChannel(messenger: messenger, window: nil)
  }

  private func makeChannel(
    messenger: FlutterBinaryMessenger,
    window: Window?
  ) -> FlutterMethodChannel {
    let channel = FlutterMethodChannel(
      name: NoteWindows.channelName,
      binaryMessenger: messenger
    )
    channel.setMethodCallHandler { [weak self, weak window] call, result in
      self?.handle(call, from: window, result: result)
    }
    return channel
  }

  private func handle(_ call: FlutterMethodCall, from window: Window?, result: @escaping FlutterResult) {
    let arguments = call.arguments as? [String: Any] ?? [:]
    switch call.method {
    case "supported":
      result(UIApplication.shared.supportsMultipleScenes)
    case "open":
      guard let noteId = arguments["noteId"] as? String, !noteId.isEmpty else {
        result(FlutterError(code: "invalid", message: "A window needs a note.", details: nil))
        return
      }
      open(
        noteId: noteId,
        environment: arguments["environment"] as? String ?? "production",
        libraryFolder: arguments["libraryFolder"] as? String ?? "",
        result: result
      )
    case "closeThisWindow":
      // The main window is never closed this way.
      if let window = window {
        close(window)
      }
      result(nil)
    case "closeWindowOf":
      if let noteId = arguments["noteId"] as? String, let target = windows[noteId] {
        close(target)
      }
      result(nil)
    case "openWindows":
      result(
        windows.values.map { window -> [String: Any] in
          ["noteId": window.noteId, "title": window.scene?.title ?? ""]
        })
    case "setTitle":
      if let window = window, let title = arguments["title"] as? String {
        window.scene?.title = title
      }
      result(nil)
    case "hand":
      hand(arguments, result: result)
    default:
      result(FlutterMethodNotImplemented)
    }
  }

  // MARK: Opening and closing

  /// Brings the note's scene forward, or asks the system for a new one.
  private func open(
    noteId: String,
    environment: String,
    libraryFolder: String,
    result: @escaping FlutterResult
  ) {
    guard UIApplication.shared.supportsMultipleScenes else {
      result(FlutterError(code: "unsupported", message: "This device shows one window.", details: nil))
      return
    }
    if let existing = windows[noteId], let session = existing.session,
      UIApplication.shared.openSessions.contains(session)
    {
      UIApplication.shared.requestSceneSessionActivation(
        session, userActivity: nil, options: nil, errorHandler: nil)
      result(nil)
      return
    }
    if pending.contains(noteId) {
      result(nil)
      return
    }
    pending.insert(noteId)
    let window = Window(noteId: noteId, environment: environment, libraryFolder: libraryFolder)
    // The error handler runs only on failure; the success is the scene
    // connecting, which clears `pending` there.
    UIApplication.shared.requestSceneSessionActivation(
      nil, userActivity: window.activity, options: nil
    ) { [weak self] _ in
      self?.pending.remove(noteId)
    }
    result(nil)
  }

  private func close(_ window: Window) {
    guard let session = window.session else { return }
    UIApplication.shared.requestSceneSessionDestruction(session, options: nil, errorHandler: nil)
  }

  /// Hands a request of a note's window to the main window, and brings it
  /// forward.
  private func hand(_ request: [String: Any], result: @escaping FlutterResult) {
    mainChannel?.invokeMethod("request", arguments: request)
    if let main = mainSession {
      UIApplication.shared.requestSceneSessionActivation(
        main, userActivity: nil, options: nil, errorHandler: nil)
    }
    result(nil)
  }

  // MARK: Scenes

  /// The main scene connected.
  func mainSceneConnected(_ session: UISceneSession) {
    mainSession = session
  }

  /// Whether a scene that is connecting is a note's: it was asked for by one
  /// of ours, or it is one that was left open and comes back.
  static func isNoteScene(session: UISceneSession, options: UIScene.ConnectionOptions) -> Bool {
    if options.userActivities.contains(where: { $0.activityType == activityType }) {
      return true
    }
    return session.stateRestorationActivity?.activityType == activityType
  }

  /// What a note's scene is started with: from the activity that asked for
  /// it, or the one it was left with.
  static func window(session: UISceneSession, options: UIScene.ConnectionOptions) -> Window? {
    let activity =
      options.userActivities.first(where: { $0.activityType == activityType })
      ?? session.stateRestorationActivity
    guard let info = activity?.userInfo,
      let noteId = info[noteIdKey] as? String, !noteId.isEmpty
    else {
      return nil
    }
    return Window(
      noteId: noteId,
      environment: info[environmentKey] as? String ?? "production",
      libraryFolder: info[libraryFolderKey] as? String ?? ""
    )
  }

  /// A note's scene connected: starts its engine on the note.
  func sceneConnected(_ window: Window, session: UISceneSession, scene: UIScene) -> FlutterEngine {
    pending.remove(window.noteId)
    let options = FlutterEngineGroupOptions()
    options.entrypoint = NoteWindows.entryPoint
    options.libraryURI = NoteWindows.entryLibrary
    options.entrypointArgs = window.entrypointArgs
    let engine = group.makeEngine(with: options)
    GeneratedPluginRegistrant.register(with: engine)
    window.engine = engine
    window.session = session
    window.scene = scene
    window.channel = makeChannel(messenger: engine.binaryMessenger, window: window)
    windows[window.noteId] = window
    return engine
  }

  /// A note's scene went away (closed, or the system let go of it).
  func sceneDisconnected(_ window: Window) {
    if windows[window.noteId] === window {
      windows[window.noteId] = nil
    }
    window.channel?.setMethodCallHandler(nil)
    window.channel = nil
    window.engine?.destroyContext()
    window.engine = nil
  }
}
