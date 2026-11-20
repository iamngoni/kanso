import AVFoundation
import Speech

// DictationTranscriber arrived with the Swift 6.2/iOS 26 SDK and lives in
// Speech, not a SpeechAnalyzer module. Keep older Xcode builds on the iOS 17
// SFSpeechRecognizer path.
#if canImport(SpeechAnalyzer) || compiler(>=6.2)
/// iOS 26's streaming dictation path. All lifecycle state stays on the main
/// actor; the audio tap captures only its input continuation and converter.
@available(iOS 26.0, *)
@MainActor
final class SpeechAnalyzerDictationSession {
  private let locale: Locale
  private let emit: (DictationEvent) -> Void
  private let eventGate = DictationEventGate()
  private let engine = AVAudioEngine()

  private var transcriber: DictationTranscriber?
  private var analyzer: SpeechAnalyzer?
  private var input: AsyncStream<AnalyzerInput>.Continuation?
  private var resultsTask: Task<Void, Never>?
  private var finishTask: Task<Void, Never>?
  private var finishTimeout: Task<Void, Never>?
  private var interruption: NSObjectProtocol?
  private var mediaReset: NSObjectProtocol?
  private var tapInstalled = false
  private var stopping = false
  private var analyzerFinalized = false
  private(set) var finished = false
  private(set) var failureCode: DictationErrorCode?

  private static let resultGracePeriod: Duration = .milliseconds(300)
  private static let finishTimeout: Duration = .seconds(3)

  init(locale: Locale, emit: @escaping (DictationEvent) -> Void) {
    self.locale = locale
    self.emit = emit
  }

  func begin() async throws {
    let preset = DictationTranscriber.Preset.progressiveLongDictation
    let reportingOptions = preset.reportingOptions.union([
      .volatileResults,
      .frequentFinalization,
    ])
    let transcriber = DictationTranscriber(
      locale: locale,
      contentHints: preset.contentHints,
      transcriptionOptions: preset.transcriptionOptions,
      reportingOptions: reportingOptions,
      attributeOptions: preset.attributeOptions
    )
    self.transcriber = transcriber
    do {
      try await Self.ensureAssets(for: transcriber)
    } catch {
      failStart(.notAvailable, message: "\(error)")
      throw error
    }
    guard !finished, !stopping else { return }

    guard let format = await SpeechAnalyzer.bestAvailableAudioFormat(compatibleWith: [transcriber])
    else {
      let error = DictationStartFailure.audioFormatUnavailable
      failStart(.audio, message: "\(error)")
      throw DictationStartFailure.audioFormatUnavailable
    }
    guard !finished, !stopping else { return }

    let analyzer = SpeechAnalyzer(modules: [transcriber])
    self.analyzer = analyzer
    let (stream, continuation) = AsyncStream<AnalyzerInput>.makeStream()
    input = continuation
    resultsTask = Task { @MainActor [weak self, transcriber] in
      do {
        for try await result in transcriber.results {
          guard let self, !self.finished else { return }
          let text = String(result.text.characters)
          guard !text.isEmpty else { continue }
          self.emit(DictationEvent(kind: result.isFinal ? .finalResult : .partial, text: text))
        }
      } catch {
        guard let self, !self.stopping, !self.finished else { return }
        self.emit(DictationEvent(kind: .error, error: .unknown, message: "\(error)"))
        self.end()
      }
    }

    do {
      try await analyzer.start(inputSequence: stream)
      guard !finished, !stopping else { return }
      try startMicrophone(format: format, continuation: continuation)
    } catch {
      guard !finished else { return }
      failureCode = .audio
      continuation.finish()
      input = nil
      await analyzer.cancelAndFinishNow()
      failStart(failureCode ?? .audio, message: "\(error)")
      throw error
    }
    observeSystem()
    emit(DictationEvent(kind: .listening))
  }

  /// Stops capture, closes input, and gives SpeechAnalyzer time to finalize
  /// its remaining result before the single terminal event.
  func stop() {
    guard !finished, !stopping else { return }
    stopping = true
    closeMicrophone()
    input?.finish()
    input = nil
    finishTask = Task { @MainActor [weak self] in
      guard let self else { return }
      do {
        try await self.analyzer?.finalizeAndFinishThroughEndOfInput()
        self.analyzerFinalized = true
      } catch {
        guard !self.finished else { return }
        self.emit(DictationEvent(kind: .error, error: .unknown, message: "\(error)"))
      }
      guard !self.finished else { return }
      let resultsTask = self.resultsTask
      await withTaskGroup(of: Void.self) { group in
        if let resultsTask {
          group.addTask { await resultsTask.value }
        }
        group.addTask { try? await Task.sleep(for: Self.resultGracePeriod) }
        _ = await group.next()
        group.cancelAll()
      }
      self.end()
    }
    finishTimeout = Task { @MainActor [weak self] in
      try? await Task.sleep(for: Self.finishTimeout)
      guard !Task.isCancelled else { return }
      self?.end()
    }
  }

  /// Cancels capture and recognition without waiting for result finalization.
  func cancel() {
    guard !finished else { return }
    stopping = true
    finishTask?.cancel()
    finishTimeout?.cancel()
    closeMicrophone()
    input?.finish()
    input = nil
    end()
  }

  private func startMicrophone(
    format: AVAudioFormat,
    continuation: AsyncStream<AnalyzerInput>.Continuation
  ) throws {
    let audio = AVAudioSession.sharedInstance()
    try audio.setCategory(.record, mode: .measurement, options: [.duckOthers])
    try audio.setActive(true, options: .notifyOthersOnDeactivation)

    let inputNode = engine.inputNode
    let microphoneFormat = inputNode.outputFormat(forBus: 0)
    let converter = DictationAudioBufferConverter()
    let throttle = LevelThrottle()
    let emit = self.emit
    let eventGate = self.eventGate
    inputNode.installTap(onBus: 0, bufferSize: 1024, format: microphoneFormat) { [weak self] buffer, _ in
      do {
        let converted = try converter.convert(buffer, to: format)
        continuation.yield(AnalyzerInput(buffer: converted))
      } catch {
        if eventGate.claimAudioFailure() {
          let message = "\(error)"
          Task { @MainActor [weak self] in
            guard let self, !self.finished else { return }
            self.emit(DictationEvent(kind: .error, error: .audio, message: message))
            self.stop()
          }
        }
      }
      if let level = throttle.next(DictationLevel.of(buffer)) {
        DispatchQueue.main.async {
          guard eventGate.isOpen else { return }
          emit(DictationEvent(kind: .level, level: level))
        }
      }
    }
    tapInstalled = true
    engine.prepare()
    try engine.start()
  }

  private func observeSystem() {
    let center = NotificationCenter.default
    let audio = AVAudioSession.sharedInstance()
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
    if engine.isRunning { engine.stop() }
    if tapInstalled {
      engine.inputNode.removeTap(onBus: 0)
      tapInstalled = false
    }
  }

  private func teardown() {
    finishTask?.cancel()
    finishTimeout?.cancel()
    resultsTask?.cancel()
    resultsTask = nil
    closeMicrophone()
    for token in [interruption, mediaReset] {
      if let token { NotificationCenter.default.removeObserver(token) }
    }
    interruption = nil
    mediaReset = nil
    try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)
    analyzer = nil
    transcriber = nil
  }

  private func end() {
    guard !finished else { return }
    finished = true
    stopping = true
    eventGate.close()
    if !analyzerFinalized, let analyzer {
      Task { await analyzer.cancelAndFinishNow() }
    }
    input?.finish()
    input = nil
    teardown()
    emit(DictationEvent(kind: .ended))
  }

  private func failStart(_ code: DictationErrorCode, message: String) {
    guard !finished else { return }
    failureCode = code
    finished = true
    stopping = true
    eventGate.close()
    teardown()
    emit(DictationEvent(kind: .error, error: code, message: message))
    emit(DictationEvent(kind: .ended))
  }

  private static func ensureAssets(for transcriber: DictationTranscriber) async throws {
    let modules: [any SpeechModule] = [transcriber]
    guard await AssetInventory.status(forModules: modules) != .installed else { return }
    if let request = try await AssetInventory.assetInstallationRequest(supporting: modules) {
      try await request.downloadAndInstall()
    }
    guard await AssetInventory.status(forModules: modules) == .installed else {
      throw DictationStartFailure.assetsUnavailable
    }
  }
}

private final class DictationEventGate: @unchecked Sendable {
  private let lock = NSLock()
  private var open = true
  private var audioFailureReported = false

  var isOpen: Bool {
    lock.lock()
    defer { lock.unlock() }
    return open
  }

  func close() {
    lock.lock()
    open = false
    lock.unlock()
  }

  func claimAudioFailure() -> Bool {
    lock.lock()
    defer { lock.unlock() }
    guard open, !audioFailureReported else { return false }
    audioFailureReported = true
    return true
  }
}

private enum DictationStartFailure: Error {
  case assetsUnavailable
  case audioFormatUnavailable
}

#endif

