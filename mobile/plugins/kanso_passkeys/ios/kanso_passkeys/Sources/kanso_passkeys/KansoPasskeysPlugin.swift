import Flutter
import UIKit

/// The platform's passkey sheet. The answer goes to Rust (`kanso_passkey_answer`);
/// Dart is told only how the ceremony ended.
public class KansoPasskeysPlugin: NSObject, FlutterPlugin, PasskeysHostApi {
  public static func register(with registrar: FlutterPluginRegistrar) {
    let instance = KansoPasskeysPlugin()
    PasskeysHostApiSetup.setUp(binaryMessenger: registrar.messenger(), api: instance)
  }

  func run(ticket: Int64, kind: String, optionsJson: String) async throws -> PasskeyOutcome {
    guard let ticket = UInt64(exactly: ticket) else { return .failed }
    let register: Bool
    switch kind.lowercased() {
    case "register": register = true
    case "assert": register = false
    default: return await refuse(ticket, "failed", .failed)
    }
    guard
      let options = PasskeyOptions(json: optionsJson, register: register),
      let ceremony = await MainActor.run(body: { Ceremony(options: options, register: register) })
    else { return await refuse(ticket, "failed", .failed) }

    let answer = await ceremony.run()
    let accepted = RustAnswer.deliver(ticket: ticket, json: answer)
    switch Self.reason(of: answer) {
    case nil: return accepted ? .done : .failed
    case "cancelled": return .cancelled
    case "unavailable": return .unavailable
    default: return .failed
    }
  }

  /// Tells Rust the ceremony could not run, so it never waits on it.
  private func refuse(_ ticket: UInt64, _ reason: String, _ outcome: PasskeyOutcome) async -> PasskeyOutcome {
    _ = RustAnswer.deliver(ticket: ticket, json: PasskeyAnswer.failure(reason))
    return outcome
  }

  /// The `error` of a failure answer, nil for a real one.
  private static func reason(of answer: String) -> String? {
    guard
      let data = answer.data(using: .utf8),
      let object = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any]
    else { return "failed" }
    return object["error"] as? String
  }
}
