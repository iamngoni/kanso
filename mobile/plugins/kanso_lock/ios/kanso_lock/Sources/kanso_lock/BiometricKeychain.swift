import Foundation
import LocalAuthentication
import Security

/// The biometric secret `B`: a Keychain generic password that opens only after
/// a biometric check and is deleted by the system when the enrolled
/// biometrics change or the passcode is removed.
enum BiometricKeychain {
  static let service = "app.kanso.Kanso.biometric"
  static let secretLength = 32

  private static func base(_ libraryId: String) -> [String: Any] {
    [
      kSecClass as String: kSecClassGenericPassword,
      kSecAttrService as String: service,
      kSecAttrAccount as String: libraryId,
    ]
  }

  /// Stores `secret` for `libraryId`, replacing any earlier one. `context`
  /// has passed the biometric check, so there is no second prompt.
  static func store(_ secret: [UInt8], libraryId: String, context: LAContext) -> OSStatus {
    var error: Unmanaged<CFError>?
    guard
      let access = SecAccessControlCreateWithFlags(
        nil,
        kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly,
        .biometryCurrentSet,
        &error
      )
    else {
      return errSecParam
    }
    _ = SecItemDelete(base(libraryId) as CFDictionary)
    var item = base(libraryId)
    item[kSecValueData as String] = Data(secret)
    item[kSecAttrAccessControl as String] = access
    item[kSecUseAuthenticationContext as String] = context
    return SecItemAdd(item as CFDictionary, nil)
  }

  /// Reads the secret through `context` (already checked, so no prompt).
  static func read(libraryId: String, context: LAContext) -> (OSStatus, [UInt8]?) {
    var query = base(libraryId)
    query[kSecReturnData as String] = true
    query[kSecMatchLimit as String] = kSecMatchLimitOne
    query[kSecUseAuthenticationContext as String] = context
    var result: CFTypeRef?
    let status = SecItemCopyMatching(query as CFDictionary, &result)
    guard status == errSecSuccess, let data = result as? Data else { return (status, nil) }
    return (status, [UInt8](data))
  }

  /// Deletes the item for `libraryId`; no error when there is none.
  static func delete(libraryId: String) {
    _ = SecItemDelete(base(libraryId) as CFDictionary)
  }

  /// `count` random bytes, nil when the system has none to give.
  static func randomBytes(_ count: Int) -> [UInt8]? {
    var bytes = [UInt8](repeating: 0, count: count)
    let status = bytes.withUnsafeMutableBytes {
      SecRandomCopyBytes(kSecRandomDefault, count, $0.baseAddress!)
    }
    return status == errSecSuccess ? bytes : nil
  }
}

/// Overwrites `bytes` with zeros in a way the optimizer keeps.
func zeroize(_ bytes: inout [UInt8]) {
  bytes.withUnsafeMutableBytes { raw in
    guard let base = raw.baseAddress else { return }
    memset_s(base, raw.count, 0, raw.count)
  }
}
