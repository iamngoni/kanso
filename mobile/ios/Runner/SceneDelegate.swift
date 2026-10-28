import Flutter
import UIKit

class SceneDelegate: FlutterSceneDelegate {
  // Documents other apps open in Kanso arrive here: with the launch, or
  // while the app runs. `KansoFiles` copies and queues the files.
  override func scene(
    _ scene: UIScene,
    willConnectTo session: UISceneSession,
    options connectionOptions: UIScene.ConnectionOptions
  ) {
    super.scene(scene, willConnectTo: session, options: connectionOptions)
    KansoFiles.shared.open(connectionOptions.urlContexts.map { $0.url })
  }

  override func scene(_ scene: UIScene, openURLContexts URLContexts: Set<UIOpenURLContext>) {
    super.scene(scene, openURLContexts: URLContexts)
    KansoFiles.shared.open(URLContexts.map { $0.url })
  }
}
