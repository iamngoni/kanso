import Foundation
import Speech

/// Recognizes the words in an audio file with the system's recognizer.
///
/// Audio stays on the device unless the caller allowed the server: a language
/// whose recognizer can't run on the device then answers `notOnDevice` and
/// nothing is sent anywhere.
enum FileTranscriber {
  /// Why a file wasn't recognized (the `error` strings of `FileTranscription`).
  enum Failure: String, Error {
    case unsupported
    case notOnDevice
    case denied
    case unreadable
    case failed
  }

  static func transcribe(path: String, localeTag: String?, allowServer: Bool) async
    -> FileTranscription
  {
    guard FileManager.default.isReadableFile(atPath: path) else {
      return fail(.unreadable)
    }
    guard await authorized() else { return fail(.denied) }

    let locale = localeTag.map { Locale(identifier: $0) } ?? Locale.current
    guard let recognizer = SFSpeechRecognizer(locale: locale), recognizer.isAvailable else {
      return fail(.unsupported)
    }
    let onDevice = recognizer.supportsOnDeviceRecognition
    if !onDevice && !allowServer { return fail(.notOnDevice) }

    let request = SFSpeechURLRecognitionRequest(url: URL(fileURLWithPath: path))
    request.requiresOnDeviceRecognition = onDevice
    request.shouldReportPartialResults = false

    let outcome = await recognize(recognizer, request)
    switch outcome {
    case .success(let text):
      return FileTranscription(text: text, onDevice: onDevice, error: nil)
    case .failure(let failure):
      return fail(failure)
    }
  }

  private static func fail(_ failure: Failure) -> FileTranscription {
    FileTranscription(text: "", onDevice: false, error: failure.rawValue)
  }

  /// Speech recognition must be allowed; asked for once if not yet decided.
  private static func authorized() async -> Bool {
    if SFSpeechRecognizer.authorizationStatus() == .notDetermined {
      await withCheckedContinuation { (done: CheckedContinuation<Void, Never>) in
        SFSpeechRecognizer.requestAuthorization { _ in done.resume() }
      }
    }
    return SFSpeechRecognizer.authorizationStatus() == .authorized
  }

  /// Runs the request to its final result.
  private static func recognize(
    _ recognizer: SFSpeechRecognizer, _ request: SFSpeechURLRecognitionRequest
  ) async -> Result<String, Failure> {
    await withCheckedContinuation { (done: CheckedContinuation<Result<String, Failure>, Never>) in
      let once = Once()
      // Held until the final result so the task isn't released early.
      var task: SFSpeechRecognitionTask?
      task = recognizer.recognitionTask(with: request) { result, error in
        if let result, result.isFinal {
          once.run {
            done.resume(returning: .success(result.bestTranscription.formattedString))
          }
        } else if let error {
          once.run {
            done.resume(returning: isNoSpeech(error) ? .success("") : .failure(.failed))
          }
        }
        if result?.isFinal == true || error != nil { task = nil }
      }
      if task == nil { once.run { done.resume(returning: .failure(.failed)) } }
    }
  }

  /// The recognizer's "no speech detected" error: nothing was heard, which is
  /// a result, not a failure.
  private static func isNoSpeech(_ error: Error) -> Bool {
    let nsError = error as NSError
    return nsError.domain == "kAFAssistantErrorDomain" && nsError.code == 1110
  }

  /// Runs a closure at most once, from any thread.
  private final class Once: @unchecked Sendable {
    private let lock = NSLock()
    private var ran = false

    func run(_ body: () -> Void) {
      lock.lock()
      let first = !ran
      ran = true
      lock.unlock()
      if first { body() }
    }
  }
}
