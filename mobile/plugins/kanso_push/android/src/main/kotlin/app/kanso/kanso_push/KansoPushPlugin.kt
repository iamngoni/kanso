// Kanso — KansoPushPlugin.kt
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

package app.kanso.kanso_push

import android.content.Context
import android.os.Handler
import android.os.Looper
import com.google.firebase.FirebaseApp
import com.google.firebase.messaging.FirebaseMessaging
import io.flutter.embedding.engine.plugins.FlutterPlugin
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

/**
 * Gets the FCM token and passes it to Dart. Firebase is optional: a build
 * without `google-services.json` has no Firebase app, and then push is
 * unavailable and nothing here runs.
 */
class KansoPushPlugin : FlutterPlugin, KansoPushHostApi {
    private var context: Context? = null

    override fun onAttachedToEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        context = binding.applicationContext
        KansoPushHostApi.setUp(binding.binaryMessenger, this)
        flutterApi = KansoPushFlutterApi(binding.binaryMessenger)
    }

    override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        KansoPushHostApi.setUp(binding.binaryMessenger, null)
        flutterApi = null
        context = null
    }

    override fun unavailableReason(): String? {
        val app = context ?: return "The plugin is not attached."
        return if (firebaseReady(app)) null else NO_FIREBASE
    }

    override fun register() {
        val app = context ?: return
        if (!firebaseReady(app)) {
            return
        }
        // A token that arrived while no engine was there comes first.
        takeStashed(app)?.let { deliver(it) }
        try {
            FirebaseMessaging.getInstance().token.addOnSuccessListener { token ->
                if (!token.isNullOrEmpty()) {
                    deliver(token)
                }
            }
        } catch (_: Throwable) {
            // Push is a convenience; a failure here must never crash the app.
        }
    }

    companion object {
        private const val PREFERENCES = "app.kanso.kanso_push"
        private const val PENDING_TOKEN = "pending_token"
        private const val NO_FIREBASE =
            "This build has no Firebase configuration (google-services.json)."

        @Volatile
        private var flutterApi: KansoPushFlutterApi? = null

        private val main = Handler(Looper.getMainLooper())

        /** Whether this build has a Firebase app to get a token from. */
        private fun firebaseReady(context: Context): Boolean =
            try {
                FirebaseApp.getApps(context).isNotEmpty() ||
                    FirebaseApp.initializeApp(context) != null
            } catch (_: Throwable) {
                false
            }

        /** The OS issued a new [token]: tell Dart, or keep it until it asks. */
        internal fun tokenArrived(context: Context, token: String) {
            if (flutterApi == null) {
                stash(context, token)
            } else {
                deliver(token) { stash(context, token) }
            }
        }

        private fun deliver(token: String, onFailure: () -> Unit = {}) {
            main.post {
                val api = flutterApi
                if (api == null) {
                    onFailure()
                    return@post
                }
                CoroutineScope(Dispatchers.Main).launch {
                    try {
                        api.onToken("fcm", token, "production")
                    } catch (_: Throwable) {
                        onFailure()
                    }
                }
            }
        }

        private fun stash(context: Context, token: String) {
            context.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE)
                .edit()
                .putString(PENDING_TOKEN, token)
                .apply()
        }

        private fun takeStashed(context: Context): String? {
            val preferences = context.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE)
            val token = preferences.getString(PENDING_TOKEN, null)
            if (token != null) {
                preferences.edit().remove(PENDING_TOKEN).apply()
            }
            return token
        }
    }
}
