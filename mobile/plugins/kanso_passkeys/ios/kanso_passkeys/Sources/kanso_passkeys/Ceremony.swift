import AuthenticationServices
import CryptoKit
import UIKit

/// One passkey ceremony: the sheet, and what it answered.
///
/// Main thread only. The answer is the contract's JSON (or a failure);
/// nothing is returned that carries a secret.
@MainActor
final class Ceremony: NSObject, ASAuthorizationControllerDelegate,
  ASAuthorizationControllerPresentationContextProviding
{
  private let controller: ASAuthorizationController
  private let register: Bool
  private var continuation: CheckedContinuation<String, Never>?

  /// Nil when the request can't be made.
  init?(options: PasskeyOptions, register: Bool) {
    let provider = ASAuthorizationPlatformPublicKeyCredentialProvider(
      relyingPartyIdentifier: options.rpId)
    let request: ASAuthorizationRequest
    if register {
      guard let userId = options.userId, let username = options.username else { return nil }
      let made = provider.createCredentialRegistrationRequest(
        challenge: options.challenge, name: username, userID: userId)
      made.userVerificationPreference = .required
      made.attestationPreference = .none
      // Before iOS 17.4 the sheet can't be told about existing credentials;
      // the server refuses a duplicate in any case.
      if #available(iOS 17.4, *) {
        made.excludedCredentials = options.credentials.map {
          ASAuthorizationPlatformPublicKeyCredentialDescriptor(credentialID: $0)
        }
      }
      if #available(iOS 18.0, *), let input = options.prfInput {
        made.prf = .inputValues(.init(saltInput1: input))
      }
      request = made
    } else {
      let made = provider.createCredentialAssertionRequest(challenge: options.challenge)
      made.userVerificationPreference = .required
      made.allowedCredentials = options.credentials.map {
        ASAuthorizationPlatformPublicKeyCredentialDescriptor(credentialID: $0)
      }
      if #available(iOS 18.0, *), let input = options.prfInput {
        made.prf = .inputValues(.init(saltInput1: input))
      }
      request = made
    }
    controller = ASAuthorizationController(authorizationRequests: [request])
    self.register = register
    super.init()
    controller.delegate = self
    controller.presentationContextProvider = self
  }

  /// Shows the sheet and waits for it. The result is the answer JSON.
  func run() async -> String {
    await withCheckedContinuation { continuation in
      self.continuation = continuation
      controller.performRequests()
    }
  }

  private func finish(_ json: String) {
    continuation?.resume(returning: json)
    continuation = nil
  }

  // MARK: ASAuthorizationControllerDelegate

  func authorizationController(
    controller: ASAuthorizationController, didCompleteWithAuthorization authorization: ASAuthorization
  ) {
    switch authorization.credential {
    case let credential as ASAuthorizationPlatformPublicKeyCredentialRegistration:
      guard let attestation = credential.rawAttestationObject else {
        finish(PasskeyAnswer.failure("failed"))
        return
      }
      var prf: Data?
      if #available(iOS 18.0, *), credential.prf?.isSupported == true {
        prf = Self.bytes(credential.prf?.first)
      }
      finish(
        PasskeyAnswer.registration(
          credentialId: credential.credentialID, attestationObject: attestation,
          clientDataJSON: credential.rawClientDataJSON, prf: prf))
    case let credential as ASAuthorizationPlatformPublicKeyCredentialAssertion:
      var prf: Data?
      if #available(iOS 18.0, *) {
        prf = Self.bytes(credential.prf?.first)
      }
      finish(
        PasskeyAnswer.assertion(
          credentialId: credential.credentialID, authenticatorData: credential.rawAuthenticatorData,
          clientDataJSON: credential.rawClientDataJSON, signature: credential.signature,
          userHandle: credential.userID, prf: prf))
    default:
      finish(PasskeyAnswer.failure("failed"))
    }
  }

  /// A PRF output's bytes. The key type is only the platform's wrapper.
  private static func bytes(_ key: SymmetricKey?) -> Data? {
    key?.withUnsafeBytes { Data($0) }
  }

  func authorizationController(controller: ASAuthorizationController, didCompleteWithError error: Error) {
    let code = (error as? ASAuthorizationError)?.code
    switch code {
    case .canceled:
      finish(PasskeyAnswer.failure("cancelled"))
    case .notHandled, .notInteractive:
      finish(PasskeyAnswer.failure("unavailable"))
    default:
      finish(PasskeyAnswer.failure("failed"))
    }
  }

  // MARK: ASAuthorizationControllerPresentationContextProviding

  func presentationAnchor(for controller: ASAuthorizationController) -> ASPresentationAnchor {
    let windows = UIApplication.shared.connectedScenes
      .compactMap { $0 as? UIWindowScene }
      .flatMap { $0.windows }
    return windows.first(where: { $0.isKeyWindow }) ?? windows.first ?? ASPresentationAnchor()
  }
}
