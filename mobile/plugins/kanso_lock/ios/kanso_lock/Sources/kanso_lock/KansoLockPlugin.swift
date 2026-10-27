import Flutter
import LocalAuthentication
import UIKit

/// Biometric unlock for locked notes and the privacy cover. The biometric
/// secret goes to Rust (`kanso_biometric_answer`); Dart is told only how the
/// check ended.
public class KansoLockPlugin: NSObject, FlutterPlugin, BiometricsHostApi, PrivacyHostApi {
  public static func register(with registrar: FlutterPluginRegistrar) {
    let instance = KansoLockPlugin()
    BiometricsHostApiSetup.setUp(binaryMessenger: registrar.messenger(), api: instance)
    PrivacyHostApiSetup.setUp(binaryMessenger: registrar.messenger(), api: instance)
    MainActor.assumeIsolated { PrivacyCover.shared.start() }
  }

  // MARK: PrivacyHostApi

  func setSecure(on: Bool) throws {
    MainActor.assumeIsolated { PrivacyCover.shared.setSecure(on) }
  }

  // MARK: BiometricsHostApi

  func availability() throws -> BiometricKind {
    let context = LAContext()
    var error: NSError?
    let can = context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error)
    if !can {
      switch (error as? LAError)?.code {
      case .biometryNotEnrolled: return .notEnrolled
      default: return .none
      }
    }
    switch context.biometryType {
    case .faceID: return .faceId
    case .touchID: return .touchId
    case .opticID: return .opticId
    default: return .none
    }
  }

  func enroll(ticket: Int64, libraryId: String, reason: String) async throws -> BiometricOutcome {
    guard let ticket = UInt64(exactly: ticket) else { return .failed }
    let context = Self.context()
    do {
      _ = try await context.evaluatePolicy(
        .deviceOwnerAuthenticationWithBiometrics, localizedReason: reason)
    } catch {
      return Self.finish(ticket, BiometricStatus.of(error), nil, libraryId)
    }
    guard var secret = BiometricKeychain.randomBytes(BiometricKeychain.secretLength) else {
      return Self.finish(ticket, .failed, nil, libraryId)
    }
    defer { zeroize(&secret) }
    let stored = BiometricKeychain.store(secret, libraryId: libraryId, context: context)
    guard stored == errSecSuccess else {
      return Self.finish(ticket, BiometricStatus.ofKeychain(stored), nil, libraryId)
    }
    let outcome = Self.finish(ticket, .ok, secret, libraryId)
    if outcome != .done {
      // Rust did not keep this secret, so the item would never open its keys.
      BiometricKeychain.delete(libraryId: libraryId)
    }
    return outcome
  }

  func unlock(ticket: Int64, libraryId: String, reason: String) async throws -> BiometricOutcome {
    guard let ticket = UInt64(exactly: ticket) else { return .failed }
    let context = Self.context()
    do {
      _ = try await context.evaluatePolicy(
        .deviceOwnerAuthenticationWithBiometrics, localizedReason: reason)
    } catch {
      return Self.finish(ticket, BiometricStatus.of(error), nil, libraryId)
    }
    var (status, secret) = BiometricKeychain.read(libraryId: libraryId, context: context)
    guard var bytes = secret, status == errSecSuccess else {
      return Self.finish(ticket, BiometricStatus.ofKeychain(status), nil, libraryId)
    }
    secret = nil
    defer { zeroize(&bytes) }
    return Self.finish(ticket, .ok, bytes, libraryId)
  }

  func forget(libraryId: String) throws {
    BiometricKeychain.delete(libraryId: libraryId)
  }

  // MARK: Helpers

  private static func context() -> LAContext {
    let context = LAContext()
    context.localizedFallbackTitle = "Use password"
    return context
  }

  /// Gives Rust the answer, so it never waits, and returns what Dart is told.
  /// An invalidated answer, or a secret Rust says does not open its keys,
  /// also deletes the Keychain item.
  private static func finish(
    _ ticket: UInt64, _ status: BiometricStatus, _ secret: [UInt8]?, _ libraryId: String
  ) -> BiometricOutcome {
    let reply = RustAnswer.deliver(ticket: ticket, status: status.rawValue, secret: secret)
    let outcome = status.outcome(after: reply)
    if outcome == .invalidated {
      BiometricKeychain.delete(libraryId: libraryId)
    }
    return outcome
  }
}
