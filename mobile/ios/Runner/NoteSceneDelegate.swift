import Flutter
import UIKit

/// The scene of a note's own window: a Flutter view on an engine of its own,
/// running `noteWindowMain` on the note (see `NoteWindows`).
///
/// The scene keeps the note in its state restoration activity, so the
/// windows that were open come back at launch, as the desktop's
/// `WindowLayout::note_windows` brings them back.
class NoteSceneDelegate: UIResponder, UIWindowSceneDelegate {
  var window: UIWindow?
  private var note: NoteWindows.Window?
  private var activity: NSUserActivity?

  func scene(
    _ scene: UIScene,
    willConnectTo session: UISceneSession,
    options connectionOptions: UIScene.ConnectionOptions
  ) {
    guard let windowScene = scene as? UIWindowScene,
      let note = NoteWindows.window(session: session, options: connectionOptions)
    else {
      // A scene that is not a note's (the system made a new window) has
      // nothing to show: the main window is the app's one.
      UIApplication.shared.requestSceneSessionDestruction(session, options: nil, errorHandler: nil)
      return
    }
    self.note = note
    activity = note.activity
    let engine = NoteWindows.shared.sceneConnected(note, session: session, scene: scene)
    let controller = FlutterViewController(engine: engine, nibName: nil, bundle: nil)
    let window = UIWindow(windowScene: windowScene)
    window.rootViewController = controller
    window.makeKeyAndVisible()
    self.window = window
    // The Dart side names the scene after the note once it has read it; a
    // locked note is never named by its title.
    scene.userActivity = activity
  }

  func stateRestorationActivity(for scene: UIScene) -> NSUserActivity? {
    return activity
  }

  func sceneDidDisconnect(_ scene: UIScene) {
    if let note = note {
      NoteWindows.shared.sceneDisconnected(note)
    }
    note = nil
    window = nil
  }
}
