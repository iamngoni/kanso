import Flutter
import UIKit
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
    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
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
    registerWidgetChannel(engineBridge.pluginRegistry)
    if let registrar = engineBridge.pluginRegistry.registrar(forPlugin: "KansoFiles") {
      KansoFiles.shared.register(with: registrar.messenger())
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
}
