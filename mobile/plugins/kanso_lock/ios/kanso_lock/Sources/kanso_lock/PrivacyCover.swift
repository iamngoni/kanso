import UIKit

/// A blurred cover over the app's window while the app is not active and
/// `secure` is on, so the app switcher's snapshot shows no note text.
///
/// The plugin watches the scene's notifications itself, so the app's scene
/// delegate needs no code for it.
@MainActor
final class PrivacyCover {
  static let shared = PrivacyCover()

  private var secure = false
  private var cover: UIView?
  private var observers: [NSObjectProtocol] = []

  private init() {}

  /// Starts watching the scenes. Once.
  func start() {
    guard observers.isEmpty else { return }
    let center = NotificationCenter.default
    observers.append(
      center.addObserver(forName: UIScene.willDeactivateNotification, object: nil, queue: .main) {
        [weak self] note in
        MainActor.assumeIsolated { self?.show(over: note.object as? UIWindowScene) }
      })
    observers.append(
      center.addObserver(forName: UIScene.didActivateNotification, object: nil, queue: .main) {
        [weak self] _ in
        MainActor.assumeIsolated { self?.hide() }
      })
  }

  /// Turns the cover on or off. Off takes down a cover that is showing only
  /// while the app is active. Inactive (the app switcher is taking its
  /// snapshot), the cover stays up until the app is active again, so a
  /// `setSecure(false)` that arrives from Dart then, as the app locks, can't
  /// uncover the note before the snapshot.
  func setSecure(_ on: Bool) {
    secure = on
    if !on && UIApplication.shared.applicationState == .active { hide() }
  }

  private func show(over scene: UIWindowScene?) {
    guard secure, cover == nil else { return }
    guard let window = scene?.keyWindow ?? scene?.windows.first else { return }
    let blur = UIVisualEffectView(effect: UIBlurEffect(style: .systemMaterial))
    blur.frame = window.bounds
    blur.autoresizingMask = [.flexibleWidth, .flexibleHeight]
    blur.isUserInteractionEnabled = false
    let glyph = UIImageView(
      image: UIImage(
        systemName: "lock.fill",
        withConfiguration: UIImage.SymbolConfiguration(pointSize: 44, weight: .regular)))
    glyph.tintColor = .secondaryLabel
    glyph.translatesAutoresizingMaskIntoConstraints = false
    blur.contentView.addSubview(glyph)
    NSLayoutConstraint.activate([
      glyph.centerXAnchor.constraint(equalTo: blur.contentView.centerXAnchor),
      glyph.centerYAnchor.constraint(equalTo: blur.contentView.centerYAnchor),
    ])
    window.addSubview(blur)
    cover = blur
  }

  private func hide() {
    cover?.removeFromSuperview()
    cover = nil
  }
}
