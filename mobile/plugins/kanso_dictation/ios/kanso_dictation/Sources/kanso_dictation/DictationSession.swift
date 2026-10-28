import AVFoundation
import Speech

/// One run of the recognizer: the microphone into an on-device request,
/// until the person stops it, the system interrupts it or the recognizer
/// ends it. It reports through `emit`, and the last thing it reports, once,
/// is `ended`.
@MainActor
final class DictationSession {
  private let recognizer: SFSpeechRecognizer
  private let emit: (DictationEvent) -> Void
  private let engine = AVAudioEngine()
  private var request: SFSpeechAudioBufferRecognitionRequest?
  private var task: SFSpeechRecognitionTask?
  private var interruption: NSObjectProtocol?
  private var mediaReset: NSObjectProtocol?
  private var stopping = false
  private(set) var finished = false

  /// How long the recognizer gets to deliver its final result after the
  /// microphone closed.
  private static let finishTimeout: Duration = .seconds(3)

  init(recognizer: SFSpeechRecognizer, emit: @escaping (DictationEvent) -> Void) {
    self.recognizer = recognizer
    self.emit = emit
  }

  /// Opens the microphone and starts recognizing. Throws when the audio
  /// session or the engine cannot start; nothing is left running then.
  func begin() throws {
    let audio = AVAudioSession.sharedInstance()
    do {
      try audio.setCategory(.record, mode: .measurement, options: [.duckOthers])
      try audio.setActive(true, options: .notifyOthersOnDeactivation)

      let request = SFSpeechAudioBufferRecognitionRequest()
      request.shouldReportPartialResults = true
      // Never a server: the caller checked supportsOnDeviceRecognition.
      request.requiresOnDeviceRecognition = true
      request.addsPunctuation = true
      self.request = request

      let input = engine.inputNode
      input.installTap(
        onBus: 0,
        bufferSize: 1024,
        format: input.outputFormat(forBus: 0),
        block: DictationSession.tap(request: request, emit: emit)
      )
      engine.prepare()
      try engine.start()
    } catch {
      teardown()
      throw error
    }

    observeSystem(audio)
    task = recognizer.recognitionTask(with: request!) { [weak self] result, error in
      let text = result?.bestTranscription.formattedString
      let isFinal = result?.isFinal ?? false
      let failure = error as NSError?
      Task { @MainActor in self?.handle(text: text, isFinal: isFinal, failure: failure) }
    }
    emit(DictationEvent(kind: .listening))
  }

  /// Closes the microphone and lets the recognizer finish what it heard.
  func stop() {
    guard !finished, !stopping else { return }
    stopping = true
    closeMicrophone()
    request?.endAudio()
    Task { @MainActor [weak self] in
      try? await Task.sleep(for: DictationSession.finishTimeout)
      self?.end()
    }
  }

  /// Stops at once, dropping what the recognizer had not finished.
  func cancel() {
    guard !finished else { return }
    task?.cancel()
    end()
  }

  private func handle(text: String?, isFinal: Bool, failure: NSError?) {
    guard !finished else { return }
    if let text, !text.isEmpty {
      emit(DictationEvent(kind: isFinal ? .finalResult : .partial, text: text))
    }
    if let failure {
      // A task the person stopped ends with an error once its audio is done;
      // that is the end, not a failure.
      if !stopping {
        emit(DictationEvent(kind: .error, error: DictationSession.code(of: failure), message: "\(failure.domain) \(failure.code)"))
      }
      end()
    } else if isFinal {
      end()
    }
  }

  /// The reason for a failure of the recognition task.
  static func code(of failure: NSError) -> DictationErrorCode {
    // kAFAssistantErrorDomain 1110: no speech was detected.
    if failure.domain == "kAFAssistantErrorDomain", failure.code == 1110 {
      return .noSpeech
    }
    return .unknown
  }

  /// The tap on the microphone, made away from the session so that it holds
  /// nothing of it: it runs on the audio thread.
  nonisolated private static func tap(
    request: SFSpeechAudioBufferRecognitionRequest,
    emit: @escaping (DictationEvent) -> Void
  ) -> AVAudioNodeTapBlock {
    let throttle = LevelThrottle()
    return { buffer, _ in
      request.append(buffer)
      if let level = throttle.next(DictationLevel.of(buffer)) {
        DispatchQueue.main.async { emit(DictationEvent(kind: .level, level: level)) }
      }
    }
  }

  private func observeSystem(_ audio: AVAudioSession) {
    let center = NotificationCenter.default
    interruption = center.addObserver(
      forName: AVAudioSession.interruptionNotification, object: audio, queue: .main
    ) { [weak self] note in
      let raw = note.userInfo?[AVAudioSessionInterruptionTypeKey] as? UInt
      guard raw == AVAudioSession.InterruptionType.began.rawValue else { return }
      Task { @MainActor in
        guard let self, !self.finished else { return }
        self.emit(DictationEvent(kind: .error, error: .interrupted))
        self.stop()
      }
    }
    mediaReset = center.addObserver(
      forName: AVAudioSession.mediaServicesWereResetNotification, object: audio, queue: .main
    ) { [weak self] _ in
      Task { @MainActor in
        guard let self, !self.finished else { return }
        self.emit(DictationEvent(kind: .error, error: .audio))
        self.cancel()
      }
    }
  }

  private func closeMicrophone() {
    if engine.isRunning {
      engine.stop()
    }
    engine.inputNode.removeTap(onBus: 0)
  }

  private func teardown() {
    closeMicrophone()
    request = nil
    task = nil
    for token in [interruption, mediaReset] {
      if let token { NotificationCenter.default.removeObserver(token) }
    }
    interruption = nil
    mediaReset = nil
    try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)
  }

  private func end() {
    guard !finished else { return }
    finished = true
    request?.endAudio()
    task?.cancel()
    teardown()
    emit(DictationEvent(kind: .ended))
  }
}

/// Lets about fifteen levels a second through.
private final class LevelThrottle: @unchecked Sendable {
  private var last: CFAbsoluteTime = 0

  func next(_ level: Double) -> Double? {
    let now = CFAbsoluteTimeGetCurrent()
    guard now - last >= 1.0 / 15.0 else { return nil }
    last = now
    return level
  }
}
