import Flutter
import UIKit
import UserNotifications
import WidgetKit
import kanso_background
import kanso_push

@main
@objc class AppDelegate: FlutterAppDelegate, FlutterImplicitEngineDelegate {
  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    // The background sync's handler must be registered before launch finishes.
    KansoBackground.register()
    // A silent push launches the app, often without Dart; run the same
    // bounded native sync the background task uses and answer the OS.
    KansoPush.onWake = { done in
      Task {
        let code = await KansoBackgroundSync.run(
          budget: KansoBackgroundSync.refreshBudget)
        done(code >= 0 ? .newData : .failed)
      }
    }
    CrashDiagnostics.shared.start()
    let launched = super.application(application, didFinishLaunchingWithOptions: launchOptions)
    UNUserNotificationCenter.current().delegate = self
    return launched
  }

  /// A reminder may be shown while Kanso is in front; other notifications
  /// keep Flutter's existing handling.
  override func userNotificationCenter(
    _ center: UNUserNotificationCenter,
    willPresent notification: UNNotification,
    withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void
  ) {
    if KansoReminders.shared.isReminder(notification) {
      completionHandler([.banner, .sound, .list])
    } else {
      super.userNotificationCenter(
        center,
        willPresent: notification,
        withCompletionHandler: completionHandler
      )
    }
  }

  /// A reminder tap or action is answered once by the reminder coordinator.
  override func userNotificationCenter(
    _ center: UNUserNotificationCenter,
    didReceive response: UNNotificationResponse,
    withCompletionHandler completionHandler: @escaping () -> Void
  ) {
    if KansoReminders.shared.isReminder(response.notification) {
      KansoReminders.shared.respond(to: response, done: completionHandler)
    } else {
      super.userNotificationCenter(
        center,
        didReceive: response,
        withCompletionHandler: completionHandler
      )
    }
  }

  // A silent push wakes the app to sync, launching it in the background if
  // it isn't running. `KansoPush.receive` calls `KansoPush.onWake` and answers
  // the OS within 25 seconds whatever happens.
  override func application(
    _ application: UIApplication,
    didReceiveRemoteNotification userInfo: [AnyHashable: Any],
    fetchCompletionHandler completionHandler: @escaping (UIBackgroundFetchResult) -> Void
  ) {
    KansoPush.receive(completionHandler)
  }

  func didInitializeImplicitFlutterEngine(_ engineBridge: FlutterImplicitEngineBridge) {
    GeneratedPluginRegistrant.register(with: engineBridge.pluginRegistry)
    if let registrar = engineBridge.pluginRegistry.registrar(forPlugin: "KansoCrashDiagnostics") {
      CrashDiagnostics.shared.register(with: registrar.messenger())
    }
    if let registrar = engineBridge.pluginRegistry.registrar(forPlugin: "KansoReminders") {
      KansoReminders.shared.register(with: registrar.messenger())
    }
    registerWidgetChannel(engineBridge.pluginRegistry)
    if let registrar = engineBridge.pluginRegistry.registrar(forPlugin: "KansoFiles") {
      KansoFiles.shared.register(with: registrar.messenger())
    }
    // `app.kanso/windows`: a note in a window of its own (NoteWindows.swift).
    if let registrar = engineBridge.pluginRegistry.registrar(forPlugin: "KansoNoteWindows") {
      NoteWindows.shared.registerMain(messenger: registrar.messenger())
    }
  }

  /// `app.kanso.kanso/widgets`: the app asks for the home-screen widgets to
  /// be drawn again once it has written their colors (`widget.json`).
  private func registerWidgetChannel(_ registry: FlutterPluginRegistry) {
    guard let registrar = registry.registrar(forPlugin: "KansoWidgets") else { return }
    let channel = FlutterMethodChannel(
      name: "app.kanso.kanso/widgets", binaryMessenger: registrar.messenger())
    channel.setMethodCallHandler { call, result in
      guard call.method == "reload" else {
        result(FlutterMethodNotImplemented)
        return
      }
      WidgetCenter.shared.reloadAllTimelines()
      result(nil)
    }
  }

  /// A note's own window is a scene with its own configuration; every other
  /// scene is the main window's.
  override func application(
    _ application: UIApplication,
    configurationForConnecting connectingSceneSession: UISceneSession,
    options: UIScene.ConnectionOptions
  ) -> UISceneConfiguration {
    let name =
      NoteWindows.isNoteScene(session: connectingSceneSession, options: options)
      ? NoteWindows.configurationName : "flutter"
    return UISceneConfiguration(name: name, sessionRole: connectingSceneSession.role)
  }
}
