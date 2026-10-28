import AVFoundation

/// How loud a buffer of microphone samples is, for the pulse of the mic
/// button.
enum DictationLevel {
  /// 0 for silence (-50 dB or less) to 1 for full scale.
  static func of(_ buffer: AVAudioPCMBuffer) -> Double {
    guard let samples = buffer.floatChannelData?[0], buffer.frameLength > 0 else { return 0 }
    var sum: Float = 0
    for index in 0..<Int(buffer.frameLength) {
      sum += samples[index] * samples[index]
    }
    return level(rms: (sum / Float(buffer.frameLength)).squareRoot())
  }

  static func level(rms: Float) -> Double {
    guard rms > 0 else { return 0 }
    let decibels = 20 * log10(Double(rms))
    return min(1, max(0, (decibels + 50) / 50))
  }
}
