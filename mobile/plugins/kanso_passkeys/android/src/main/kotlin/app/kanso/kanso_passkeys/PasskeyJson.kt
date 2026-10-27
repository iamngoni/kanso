package app.kanso.kanso_passkeys

import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject

/**
 * The contract's JSON (`kanso_mobile::passkey_bridge`) and the WebAuthn JSON
 * Credential Manager speaks, both base64url without padding, so binary
 * values pass through as the text they already are.
 */
object PasskeyJson {
    /** A request that isn't the contract's shape. */
    class BadOptions(message: String) : Exception(message)

    /** The WebAuthn creation options for a `register` request. */
    @JvmStatic
    fun creationRequest(options: String): String {
        val given = parse(options)
        val rpId = text(given, "rp_id")
        val request =
            JSONObject()
                .put("challenge", text(given, "challenge"))
                .put("rp", JSONObject().put("id", rpId).put("name", "Kanso"))
                .put(
                    "user",
                    JSONObject()
                        .put("id", text(given, "user_id"))
                        .put("name", text(given, "username"))
                        .put("displayName", given.optString("display_name", text(given, "username"))),
                )
                .put(
                    "pubKeyCredParams",
                    JSONArray()
                        .put(JSONObject().put("type", "public-key").put("alg", -7))
                        .put(JSONObject().put("type", "public-key").put("alg", -257)),
                )
                .put("timeout", TIMEOUT_MS)
                .put("attestation", "none")
                .put("excludeCredentials", descriptors(given, "exclude_credentials"))
                .put(
                    "authenticatorSelection",
                    JSONObject()
                        .put("authenticatorAttachment", "platform")
                        .put("residentKey", "required")
                        .put("requireResidentKey", true)
                        .put("userVerification", "required"),
                )
        prfExtension(given)?.let { request.put("extensions", it) }
        return request.toString()
    }

    /** The WebAuthn request options for an `assert` request. */
    @JvmStatic
    fun requestOptions(options: String): String {
        val given = parse(options)
        val request =
            JSONObject()
                .put("challenge", text(given, "challenge"))
                .put("rpId", text(given, "rp_id"))
                .put("timeout", TIMEOUT_MS)
                .put("userVerification", "required")
                .put("allowCredentials", descriptors(given, "allow_credentials"))
        prfExtension(given)?.let { request.put("extensions", it) }
        return request.toString()
    }

    /** The contract's registration answer, from Credential Manager's response. */
    @JvmStatic
    fun registrationAnswer(responseJson: String): String {
        val response = JSONObject(responseJson)
        val inner = response.getJSONObject("response")
        val transports = inner.optJSONArray("transports")
        return JSONObject()
            .put("credential_id", credentialId(response))
            .put("attestation_object", inner.getString("attestationObject"))
            .put("client_data_json", inner.getString("clientDataJSON"))
            .put("transports", if (transports != null && transports.length() > 0) transports else JSONArray().put("internal"))
            .put("prf", prfOutput(response) ?: JSONObject.NULL)
            .toString()
    }

    /** The contract's assertion answer, from Credential Manager's response. */
    @JvmStatic
    fun assertionAnswer(responseJson: String): String {
        val response = JSONObject(responseJson)
        val inner = response.getJSONObject("response")
        val handle = inner.optString("userHandle", "")
        return JSONObject()
            .put("credential_id", credentialId(response))
            .put("authenticator_data", inner.getString("authenticatorData"))
            .put("client_data_json", inner.getString("clientDataJSON"))
            .put("signature", inner.getString("signature"))
            .put("user_handle", if (handle.isEmpty()) JSONObject.NULL else handle)
            .put("prf", prfOutput(response) ?: JSONObject.NULL)
            .toString()
    }

    /** `cancelled`, `unavailable` or `failed`. */
    @JvmStatic
    fun failure(reason: String): String = JSONObject().put("error", reason).toString()

    private const val TIMEOUT_MS = 300_000

    private fun parse(options: String): JSONObject =
        try {
            JSONObject(options)
        } catch (error: JSONException) {
            throw BadOptions("the request is not JSON")
        }

    private fun text(from: JSONObject, key: String): String {
        val value = from.optString(key, "")
        if (value.isEmpty()) throw BadOptions("the request has no $key")
        return value
    }

    private fun descriptors(from: JSONObject, key: String): JSONArray {
        val listed = from.optJSONArray(key) ?: return JSONArray()
        val out = JSONArray()
        for (index in 0 until listed.length()) {
            val id = listed.optString(index, "")
            if (id.isEmpty()) throw BadOptions("$key holds a bad entry")
            out.put(JSONObject().put("type", "public-key").put("id", id))
        }
        return out
    }

    /** `extensions.prf.eval.first`, when the request has a PRF input. */
    private fun prfExtension(from: JSONObject): JSONObject? {
        val input = from.optString("prf_input", "")
        if (input.isEmpty()) return null
        return JSONObject().put("prf", JSONObject().put("eval", JSONObject().put("first", input)))
    }

    private fun credentialId(response: JSONObject): String {
        val raw = response.optString("rawId", "")
        return if (raw.isNotEmpty()) raw else response.getString("id")
    }

    /** `clientExtensionResults.prf.results.first`, when the provider gave one. */
    private fun prfOutput(response: JSONObject): String? {
        val first =
            response.optJSONObject("clientExtensionResults")
                ?.optJSONObject("prf")
                ?.optJSONObject("results")
                ?.optString("first", "")
        return first?.takeIf { it.isNotEmpty() }
    }
}
