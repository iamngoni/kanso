// Kanso — VoiceCaptureController.swift
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import ActivityKit
import AVFoundation
import Foundation

/// What a run of the capture intent came to.
enum VoiceCaptureOutcome: Equatable {
  case started
  case saved
  case tooShort
  case failed(String)
}

/// Records a voice drop. One recording at a time, shared by every run of the
/// intent in this process; the "is recording" state is in the App Group so
/// that a run in another process finds it.
///
/// Live Activity: started when recording starts. On stop it is ended with the
/// "Saved to Inbox" state and dismissed by the system after a few seconds,
/// which is the simplest way to confirm a save without a notification.
///
/// This type never opens the library, reads keys or calls the core.
final class VoiceCaptureController: NSObject, AVAudioRecorderDelegate {
  static let shared = VoiceCaptureController()

  private static let stateKey = "app.kanso.voiceCapture.state"
  private static let savedLinger: TimeInterval = 5

  private let lock = NSLock()
  private var recorder: AVAudioRecorder?
  private var writer: VoiceDropWriter?

  private var defaults: UserDefaults? { UserDefaults(suiteName: KansoLibraryLocation.appGroup) }

  /// The saved state of the recording in progress, if any.
  private var savedState: VoiceRecordingState? {
    get {
      defaults?.data(forKey: Self.stateKey).flatMap {
        try? JSONDecoder().decode(VoiceRecordingState.self, from: $0)
      }
    }
    set {
      if let newValue, let data = try? JSONEncoder().encode(newValue) {
        defaults?.set(data, forKey: Self.stateKey)
      } else {
        defaults?.removeObject(forKey: Self.stateKey)
      }
    }
  }

  /// Starts a recording, or stops the one in progress.
  func toggle() -> VoiceCaptureOutcome {
    lock.lock()
    defer { lock.unlock() }
    let action = VoiceCaptureRules.decide(
      state: savedState, recorderActive: recorder?.isRecording == true)
    switch action {
    case .stop:
      return stopLocked()
    case .start:
      return startLocked()
    case .replaceStaleAndStart:
      if let stale = savedState, let location = KansoLibraryLocation.shared() {
        VoiceDropWriter.removePartial(id: stale.id, in: location.inbox)
      }
      savedState = nil
      endActivities(saved: false)
      return startLocked()
    }
  }

  /// Stops and saves the recording in progress, if there is one.
  func stop() -> VoiceCaptureOutcome? {
    lock.lock()
    defer { lock.unlock() }
    guard recorder != nil else { return nil }
    return stopLocked()
  }

  // MARK: - Start

  /// The audio session is set up and `record()` called before anything else,
  /// so the recording starts fast.
  private func startLocked() -> VoiceCaptureOutcome {
    guard AVAudioApplication.shared.recordPermission == .granted else {
      return .failed("Open Kanso once to allow the microphone.")
    }
    guard let location = KansoLibraryLocation.shared() else {
      return .failed("Kanso couldn't open its library.")
    }
    do {
      let session = AVAudioSession.sharedInstance()
      try session.setCategory(.record, mode: .default)
      try session.setActive(true)

      let writer = try VoiceDropWriter(inbox: location.inbox)
      let settings: [String: Any] = [
        AVFormatIDKey: kAudioFormatMPEG4AAC,
        AVSampleRateKey: 44_100,
        AVNumberOfChannelsKey: 1,
        AVEncoderBitRateKey: 64_000,
      ]
      let recorder = try AVAudioRecorder(url: writer.audioURL, settings: settings)
      recorder.delegate = self
      let seconds = Double(VoiceCaptureRules.maxDurationMs) / 1000
      guard recorder.record(forDuration: seconds) else {
        writer.discard()
        return .failed("Kanso couldn't start recording.")
      }
      let startedMs = Int64((Date().timeIntervalSince1970 * 1000).rounded())
      self.recorder = recorder
      self.writer = writer
      savedState = VoiceRecordingState(id: writer.id, startedMs: startedMs)
      startActivity(startedMs: startedMs)
      return .started
    } catch {
      return .failed("Kanso couldn't start recording.")
    }
  }

  // MARK: - Stop

  private func stopLocked() -> VoiceCaptureOutcome {
    guard let recorder, let writer, let state = savedState else {
      // Nothing of ours is recording; clear whatever the state says.
      savedState = nil
      endActivities(saved: false)
      return .failed("There was no recording to stop.")
    }
    let durationMs = Int64(recorder.currentTime * 1000)
    recorder.stop()
    self.recorder = nil
    self.writer = nil
    savedState = nil
    try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)

    guard VoiceCaptureRules.shouldKeep(durationMs: durationMs) else {
      writer.discard()
      endActivities(saved: false)
      return .tooShort
    }
    do {
      try writer.finish(startedMs: state.startedMs, durationMs: durationMs)
    } catch {
      writer.discard()
      endActivities(saved: false)
      return .failed("Kanso couldn't save this.")
    }
    endActivities(saved: true)
    return .saved
  }

  // MARK: - AVAudioRecorderDelegate

  /// The 30-minute limit ran out (a stop from `stop()` doesn't call this).
  func audioRecorderDidFinishRecording(_ recorder: AVAudioRecorder, successfully flag: Bool) {
    finishIfCurrent(recorder)
  }

  func audioRecorderEncodeErrorDidOccur(_ recorder: AVAudioRecorder, error: Error?) {
    finishIfCurrent(recorder)
  }

  private func finishIfCurrent(_ finished: AVAudioRecorder) {
    lock.lock()
    defer { lock.unlock() }
    guard recorder === finished else { return }
    _ = stopLocked()
  }

  // MARK: - Live Activity

  private func startActivity(startedMs: Int64) {
    guard ActivityAuthorizationInfo().areActivitiesEnabled else { return }
    let attributes = VoiceCaptureAttributes(
      startDate: Date(timeIntervalSince1970: Double(startedMs) / 1000))
    _ = try? Activity.request(
      attributes: attributes,
      content: ActivityContent(state: .init(saved: false), staleDate: nil))
  }

  private func endActivities(saved: Bool) {
    let activities = Activity<VoiceCaptureAttributes>.activities
    guard !activities.isEmpty else { return }
    let state = VoiceCaptureAttributes.ContentState(saved: saved)
    let policy: ActivityUIDismissalPolicy =
      saved ? .after(Date().addingTimeInterval(Self.savedLinger)) : .immediate
    Task {
      for activity in activities {
        await activity.end(ActivityContent(state: state, staleDate: nil), dismissalPolicy: policy)
      }
    }
  }
}
