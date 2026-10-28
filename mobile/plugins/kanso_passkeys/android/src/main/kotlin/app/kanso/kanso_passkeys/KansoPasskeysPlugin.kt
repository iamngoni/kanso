package app.kanso.kanso_passkeys

import android.app.Activity
import androidx.credentials.CreatePublicKeyCredentialRequest
import androidx.credentials.CreatePublicKeyCredentialResponse
import androidx.credentials.CredentialManager
import androidx.credentials.GetCredentialRequest
import androidx.credentials.GetPublicKeyCredentialOption
import androidx.credentials.PublicKeyCredential
import androidx.credentials.exceptions.CreateCredentialCancellationException
import androidx.credentials.exceptions.CreateCredentialException
import androidx.credentials.exceptions.CreateCredentialNoCreateOptionException
import androidx.credentials.exceptions.CreateCredentialProviderConfigurationException
import androidx.credentials.exceptions.CreateCredentialUnsupportedException
import androidx.credentials.exceptions.GetCredentialCancellationException
import androidx.credentials.exceptions.GetCredentialException
import androidx.credentials.exceptions.GetCredentialProviderConfigurationException
import androidx.credentials.exceptions.GetCredentialUnsupportedException
import androidx.credentials.exceptions.NoCredentialException
import io.flutter.embedding.engine.plugins.FlutterPlugin
import io.flutter.embedding.engine.plugins.activity.ActivityAware
import io.flutter.embedding.engine.plugins.activity.ActivityPluginBinding
import kotlinx.coroutines.CancellationException

/**
 * Hands an answer to Rust: `KansoNative.passkeyAnswer` in the app module,
 * which this plugin cannot see. `KansoApplication.onCreate` installs it.
 *
 * The answer holds the PRF output, so it goes from here to Rust and is never
 * returned to Dart, logged or kept.
 */
fun interface PasskeyAnswerEntry {
    /** 1 when a ceremony took the answer, 0 when none waits, -5 on a fault. */
    fun answer(ticket: Long, json: String): Int
}

object RustAnswer {
    @Volatile
    private var entry: PasskeyAnswerEntry? = null

    /** Installs the call into the library. */
    @JvmStatic
    fun install(entry: PasskeyAnswerEntry) {
        this.entry = entry
    }

    /** Gives [json] to the ceremony waiting on [ticket]; true when one took it. */
    fun deliver(ticket: Long, json: String): Boolean =
        try {
            entry?.answer(ticket, json) == 1
        } catch (error: Throwable) {
            false
        }
}

/**
 * The platform's passkey sheet through Credential Manager. The answer goes to
 * Rust ([RustAnswer]); Dart is told only how the ceremony ended.
 */
class KansoPasskeysPlugin :
    FlutterPlugin,
    ActivityAware,
    PasskeysHostApi {
    private var activity: Activity? = null

    override fun onAttachedToEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        PasskeysHostApi.setUp(binding.binaryMessenger, this)
    }

    override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        PasskeysHostApi.setUp(binding.binaryMessenger, null)
    }

    override fun onAttachedToActivity(binding: ActivityPluginBinding) {
        activity = binding.activity
    }

    override fun onDetachedFromActivityForConfigChanges() = onDetachedFromActivity()

    override fun onReattachedToActivityForConfigChanges(binding: ActivityPluginBinding) =
        onAttachedToActivity(binding)

    override fun onDetachedFromActivity() {
        activity = null
    }

    override suspend fun run(
        ticket: Long,
        kind: String,
        optionsJson: String,
    ): PasskeyOutcome {
        val host = activity
        if (host == null) {
            return refuse(ticket, "unavailable", PasskeyOutcome.UNAVAILABLE)
        }
        val answer: String
        try {
            answer =
                when (kind.lowercase()) {
                    "register" -> register(host, optionsJson)
                    "assert" -> assertion(host, optionsJson)
                    else -> throw PasskeyJson.BadOptions("unknown kind")
                }
        } catch (error: PasskeyJson.BadOptions) {
            return refuse(ticket, "failed", PasskeyOutcome.FAILED)
        } catch (error: CancellationException) {
            RustAnswer.deliver(ticket, PasskeyJson.failure("cancelled"))
            throw error
        } catch (error: Exception) {
            val (reason, outcome) = classify(error)
            return refuse(ticket, reason, outcome)
        }
        return if (RustAnswer.deliver(ticket, answer)) PasskeyOutcome.DONE else PasskeyOutcome.FAILED
    }

    private suspend fun register(
        host: Activity,
        options: String,
    ): String {
        val request = CreatePublicKeyCredentialRequest(PasskeyJson.creationRequest(options))
        val result = CredentialManager.create(host).createCredential(host, request)
        val response =
            result as? CreatePublicKeyCredentialResponse
                ?: throw IllegalStateException("not a public key credential")
        return PasskeyJson.registrationAnswer(response.registrationResponseJson)
    }

    private suspend fun assertion(
        host: Activity,
        options: String,
    ): String {
        val request =
            GetCredentialRequest(listOf(GetPublicKeyCredentialOption(PasskeyJson.requestOptions(options))))
        val result = CredentialManager.create(host).getCredential(host, request)
        val credential =
            result.credential as? PublicKeyCredential
                ?: throw IllegalStateException("not a public key credential")
        return PasskeyJson.assertionAnswer(credential.authenticationResponseJson)
    }

    /** Tells Rust the ceremony did not produce an answer, so it never waits on it. */
    private fun refuse(
        ticket: Long,
        reason: String,
        outcome: PasskeyOutcome,
    ): PasskeyOutcome {
        RustAnswer.deliver(ticket, PasskeyJson.failure(reason))
        return outcome
    }

    private fun classify(error: Exception): Pair<String, PasskeyOutcome> =
        when (error) {
            is CreateCredentialCancellationException,
            is GetCredentialCancellationException,
            -> "cancelled" to PasskeyOutcome.CANCELLED

            is CreateCredentialProviderConfigurationException,
            is CreateCredentialUnsupportedException,
            is CreateCredentialNoCreateOptionException,
            is GetCredentialProviderConfigurationException,
            is GetCredentialUnsupportedException,
            is NoCredentialException,
            -> "unavailable" to PasskeyOutcome.UNAVAILABLE

            is CreateCredentialException,
            is GetCredentialException,
            -> "failed" to PasskeyOutcome.FAILED

            else -> "failed" to PasskeyOutcome.FAILED
        }
}
