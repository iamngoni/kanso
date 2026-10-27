import Foundation

/// base64url without padding, as the facade's JSON carries every binary value.
enum Base64URL {
  static func encode(_ data: Data) -> String {
    data.base64EncodedString()
      .replacingOccurrences(of: "+", with: "-")
      .replacingOccurrences(of: "/", with: "_")
      .replacingOccurrences(of: "=", with: "")
  }

  static func decode(_ text: String) -> Data? {
    var base64 = text.replacingOccurrences(of: "-", with: "+").replacingOccurrences(of: "_", with: "/")
    base64 += String(repeating: "=", count: (4 - base64.count % 4) % 4)
    return Data(base64Encoded: base64)
  }
}

/// A ceremony's request, read from the facade's `options_json`.
struct PasskeyOptions {
  let rpId: String
  let challenge: Data
  /// Registration only.
  let userId: Data?
  let username: String?
  let displayName: String?
  /// Registration: the credentials already registered. Assertion: the ones
  /// that may answer (empty: any).
  let credentials: [Data]
  let prfInput: Data?

  /// Nil for text that is not a request of the contract's shape.
  init?(json: String, register: Bool) {
    guard
      let data = json.data(using: .utf8),
      let object = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any],
      let rpId = object["rp_id"] as? String, !rpId.isEmpty,
      let challenge = (object["challenge"] as? String).flatMap(Base64URL.decode)
    else { return nil }
    self.rpId = rpId
    self.challenge = challenge
    if register {
      guard
        let userId = (object["user_id"] as? String).flatMap(Base64URL.decode),
        let username = object["username"] as? String
      else { return nil }
      self.userId = userId
      self.username = username
      self.displayName = object["display_name"] as? String
    } else {
      self.userId = nil
      self.username = nil
      self.displayName = nil
    }
    let listed = (object[register ? "exclude_credentials" : "allow_credentials"] as? [String]) ?? []
    var credentials: [Data] = []
    for id in listed {
      guard let decoded = Base64URL.decode(id) else { return nil }
      credentials.append(decoded)
    }
    self.credentials = credentials
    if let input = object["prf_input"] as? String {
      guard let decoded = Base64URL.decode(input) else { return nil }
      self.prfInput = decoded
    } else {
      self.prfInput = nil
    }
  }
}

/// The answers handed to Rust, in the contract's shapes. Built here so the
/// keys are written once.
enum PasskeyAnswer {
  static func registration(
    credentialId: Data, attestationObject: Data, clientDataJSON: Data, prf: Data?
  ) -> String {
    json([
      "credential_id": Base64URL.encode(credentialId),
      "attestation_object": Base64URL.encode(attestationObject),
      "client_data_json": Base64URL.encode(clientDataJSON),
      "transports": ["internal"],
      "prf": prf.map(Base64URL.encode) as Any? ?? NSNull(),
    ])
  }

  static func assertion(
    credentialId: Data, authenticatorData: Data, clientDataJSON: Data, signature: Data,
    userHandle: Data?, prf: Data?
  ) -> String {
    json([
      "credential_id": Base64URL.encode(credentialId),
      "authenticator_data": Base64URL.encode(authenticatorData),
      "client_data_json": Base64URL.encode(clientDataJSON),
      "signature": Base64URL.encode(signature),
      "user_handle": userHandle.flatMap { $0.isEmpty ? nil : Base64URL.encode($0) } as Any? ?? NSNull(),
      "prf": prf.map(Base64URL.encode) as Any? ?? NSNull(),
    ])
  }

  /// `cancelled`, `unavailable` or `failed`.
  static func failure(_ reason: String) -> String {
    json(["error": reason])
  }

  private static func json(_ object: [String: Any]) -> String {
    guard
      let data = try? JSONSerialization.data(withJSONObject: object, options: [.withoutEscapingSlashes]),
      let text = String(data: data, encoding: .utf8)
    else { return "{\"error\":\"failed\"}" }
    return text
  }
}
