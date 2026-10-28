import AVFoundation
import Speech

/// Speech recognition and the microphone: both must be allowed to dictate.
enum DictationPermissions {
  /// Where the two permissions stand together: the worst of them.
  static func current() -> DictationPermission {
    combine(speech: speech(SFSpeechRecognizer.authorizationStatus()), microphone: microphone())
  }

  /// Asks for what is not asked yet. The microphone is asked only once
  /// speech recognition is allowed, so a refusal costs the person one
  /// prompt, not two.
  static func request() async -> DictationPermission {
    if SFSpeechRecognizer.authorizationStatus() == .notDetermined {
      await withCheckedContinuation { (done: CheckedContinuation<Void, Never>) in
        SFSpeechRecognizer.requestAuthorization { _ in done.resume() }
      }
    }
    if speech(SFSpeechRecognizer.authorizationStatus()) == .granted,
      AVAudioApplication.shared.recordPermission == .undetermined
    {
      _ = await AVAudioApplication.requestRecordPermission()
    }
    return current()
  }

  static func speech(_ status: SFSpeechRecognizerAuthorizationStatus) -> DictationPermission {
    switch status {
    case .authorized: return .granted
    case .denied: return .denied
    case .restricted: return .restricted
    case .notDetermined: return .notDetermined
    @unknown default: return .denied
    }
  }

  static func microphone() -> DictationPermission {
    switch AVAudioApplication.shared.recordPermission {
    case .granted: return .granted
    case .denied: return .denied
    case .undetermined: return .notDetermined
    @unknown default: return .denied
    }
  }

  /// Restricted beats denied beats not asked beats granted.
  static func combine(speech: DictationPermission, microphone: DictationPermission)
    -> DictationPermission
  {
    for outcome in [DictationPermission.restricted, .denied, .notDetermined] {
      if speech == outcome || microphone == outcome { return outcome }
    }
    return .granted
  }
}
