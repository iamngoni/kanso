import Flutter
import UIKit
import WidgetKit
import kanso_background

@main
@objc class AppDelegate: FlutterAppDelegate, FlutterImplicitEngineDelegate {
  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    // The background sync's handler must be registered before launch finishes.
    KansoBackground.register()
    CrashDiagnostics.shared.start()
    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
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
