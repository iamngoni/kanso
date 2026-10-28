import Flutter
import Speech
import UIKit

/// The platform's speech recognizer, on the device. Nothing is sent to a
/// server: a language with no on-device recognizer is unavailable.
public class KansoDictationPlugin: NSObject, FlutterPlugin, DictationHostApi {
  private let flutterApi: DictationFlutterApi
  private let pump: AsyncStream<DictationEvent>.Continuation
  private var session: DictationSession?

  init(messenger: FlutterBinaryMessenger) {
    flutterApi = DictationFlutterApi(binaryMessenger: messenger)
    // One consumer sends the events one after the other, in order.
    let (stream, continuation) = AsyncStream<DictationEvent>.makeStream()
    pump = continuation
    super.init()
    let api = flutterApi
    Task { @MainActor in
      for await event in stream {
        try? await api.onEvent(event: event)
      }
    }
  }

  public static func register(with registrar: FlutterPluginRegistrar) {
    let instance = KansoDictationPlugin(messenger: registrar.messenger())
    DictationHostApiSetup.setUp(binaryMessenger: registrar.messenger(), api: instance)
  }

  func availability(localeTag: String?) async throws -> DictationAvailability {
    let permission = DictationPermissions.current()
    let language = Self.languageName(for: localeTag)
    switch Self.recognizer(for: localeTag) {
    case .success:
      return DictationAvailability(
        available: true, onDevice: true, permission: permission, language: language)
    case .failure(let reason):
      return DictationAvailability(
        available: false, onDevice: false, permission: permission, language: language,
        reason: reason.reason)
    }
  }

  func requestPermission() async throws -> DictationPermission {
    await DictationPermissions.request()
  }

  func start(localeTag: String?) async throws {
    await MainActor.run { begin(localeTag: localeTag) }
  }

  func stop() throws {
    MainActor.assumeIsolated { session?.stop() }
  }

  func cancel() throws {
    MainActor.assumeIsolated { session?.cancel() }
  }

  func openSettings() throws {
    guard let url = URL(string: UIApplication.openSettingsURLString) else { return }
    DispatchQueue.main.async { UIApplication.shared.open(url) }
  }

  @MainActor
  private func begin(localeTag: String?) {
    if let running = session, !running.finished {
      refuse(.busy)
      return
    }
    guard DictationPermissions.current() == .granted else {
      refuse(.permissionDenied)
      return
    }
    guard case .success(let recognizer) = Self.recognizer(for: localeTag) else {
      refuse(.notAvailable)
      return
    }
    let started = DictationSession(recognizer: recognizer) { [pump] event in
      pump.yield(event)
    }
    do {
      try started.begin()
      session = started
    } catch {
      refuse(.audio, message: "\(error)")
    }
  }

  /// A session that never started: the error, then its end.
  private func refuse(_ code: DictationErrorCode, message: String? = nil) {
    pump.yield(DictationEvent(kind: .error, error: code, message: message))
    pump.yield(DictationEvent(kind: .ended))
  }

  /// The language's name, in the device's language.
  static func languageName(for localeTag: String?) -> String {
    let locale = localeTag.map { Locale(identifier: $0) } ?? Locale.current
    return Locale.current.localizedString(forIdentifier: locale.identifier) ?? locale.identifier
  }

  enum Unavailable: Error {
    case unsupportedLanguage
    case notOnDevice

    var reason: DictationUnavailableReason {
      switch self {
      case .unsupportedLanguage: return .unsupportedLanguage
      case .notOnDevice: return .notOnDevice
      }
    }
  }

  /// The recognizer for [localeTag], when it can run on the device.
  static func recognizer(for localeTag: String?) -> Result<SFSpeechRecognizer, Unavailable> {
    let locale = localeTag.map { Locale(identifier: $0) } ?? Locale.current
    guard let recognizer = SFSpeechRecognizer(locale: locale) else {
      return .failure(.unsupportedLanguage)
    }
    guard recognizer.supportsOnDeviceRecognition else {
      return .failure(.notOnDevice)
    }
    return .success(recognizer)
  }
}
