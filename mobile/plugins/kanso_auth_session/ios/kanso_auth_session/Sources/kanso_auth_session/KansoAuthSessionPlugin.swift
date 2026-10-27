import AuthenticationServices
import Flutter
import UIKit

/// The system's sign-in session (`ASWebAuthenticationSession`) for the Apple,
/// Google and GitHub pages.
///
/// One session at a time: opening another ends the open one as dismissed.
public class KansoAuthSessionPlugin: NSObject, FlutterPlugin, AuthSessionHostApi,
  ASWebAuthenticationPresentationContextProviding
{
  /// What ends the open session, once.
  private final class Pending {
    let session: ASWebAuthenticationSession
    private let continuation: CheckedContinuation<String?, Error>
    private var done = false

    init(session: ASWebAuthenticationSession, continuation: CheckedContinuation<String?, Error>) {
      self.session = session
      self.continuation = continuation
    }

    /// Main thread only.
    func finish(_ result: Result<String?, Error>) {
      guard !done else { return }
      done = true
      continuation.resume(with: result)
    }
  }

  /// Main thread only.
  private var pending: Pending?

  public static func register(with registrar: FlutterPluginRegistrar) {
    let instance = KansoAuthSessionPlugin()
    AuthSessionHostApiSetup.setUp(binaryMessenger: registrar.messenger(), api: instance)
  }

  func open(url: String, callbackScheme: String?) async throws -> String? {
    // Only web pages: never a file, a script or another app's scheme.
    guard
      let page = URL(string: url), let scheme = page.scheme?.lowercased(),
      scheme == "http" || scheme == "https", let host = page.host, !host.isEmpty
    else {
      throw PigeonError(code: "invalid_url", message: "Only http and https pages are opened.", details: nil)
    }
    if let callbackScheme, !Self.isScheme(callbackScheme) {
      throw PigeonError(code: "invalid_scheme", message: "The callback scheme is not a URL scheme.", details: nil)
    }
    return try await withCheckedThrowingContinuation { continuation in
      DispatchQueue.main.async {
        self.start(page, callbackScheme, continuation)
      }
    }
  }

  func close() throws {
    DispatchQueue.main.async {
      // The session's completion handler answers null.
      self.pending?.session.cancel()
    }
  }

  private func start(
    _ page: URL, _ callbackScheme: String?, _ continuation: CheckedContinuation<String?, Error>
  ) {
    if let earlier = pending {
      pending = nil
      earlier.session.cancel()
      earlier.finish(.success(nil))
    }
    var current: Pending?
    let session = ASWebAuthenticationSession(url: page, callbackURLScheme: callbackScheme) {
      [weak self] callback, error in
      DispatchQueue.main.async {
        guard let current else { return }
        if self?.pending === current { self?.pending = nil }
        if let error {
          if (error as? ASWebAuthenticationSessionError)?.code == .canceledLogin {
            current.finish(.success(nil))
          } else {
            current.finish(
              .failure(PigeonError(code: "failed", message: "The sign-in session failed.", details: nil)))
          }
        } else {
          current.finish(.success(callback?.absoluteString))
        }
      }
    }
    session.presentationContextProvider = self
    session.prefersEphemeralWebBrowserSession = false
    let entry = Pending(session: session, continuation: continuation)
    current = entry
    pending = entry
    if !session.start() {
      pending = nil
      entry.finish(
        .failure(PigeonError(code: "unavailable", message: "The sign-in session could not start.", details: nil)))
    }
  }

  /// A URL scheme: a letter, then letters, digits, `+`, `-` and `.`.
  static func isScheme(_ text: String) -> Bool {
    guard let first = text.unicodeScalars.first, first.isASCII, first.properties.isAlphabetic else {
      return false
    }
    return text.unicodeScalars.allSatisfy {
      $0.isASCII && ($0.properties.isAlphabetic || ("0"..."9").contains($0) || "+-.".unicodeScalars.contains($0))
    }
  }

  public func presentationAnchor(for session: ASWebAuthenticationSession) -> ASPresentationAnchor {
    let windows = UIApplication.shared.connectedScenes
      .compactMap { $0 as? UIWindowScene }
      .flatMap { $0.windows }
    return windows.first(where: { $0.isKeyWindow }) ?? windows.first ?? ASPresentationAnchor()
  }
}
