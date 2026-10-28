import Flutter
import UIKit

/// The Dart side of the background sync: `KansoBackground.runNow` and
/// `schedule`. The task itself is registered by `KansoBackground.register()`.
public class KansoBackgroundPlugin: NSObject, FlutterPlugin, BackgroundHostApi {
  public static func register(with registrar: FlutterPluginRegistrar) {
    let instance = KansoBackgroundPlugin()
    BackgroundHostApiSetup.setUp(binaryMessenger: registrar.messenger(), api: instance)
  }

  func runNow() async throws -> Int64 {
    Int64(await KansoBackgroundSync.run(budget: KansoBackgroundSync.refreshBudget))
  }

  func schedule() throws {
    KansoBackground.schedule()
  }
}
