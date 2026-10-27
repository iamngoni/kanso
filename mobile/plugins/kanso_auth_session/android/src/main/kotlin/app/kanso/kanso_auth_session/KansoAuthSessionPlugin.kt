package app.kanso.kanso_auth_session

import android.app.Activity
import android.app.Application
import android.net.Uri
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import androidx.browser.customtabs.CustomTabsIntent
import io.flutter.embedding.engine.plugins.FlutterPlugin
import io.flutter.embedding.engine.plugins.activity.ActivityAware
import io.flutter.embedding.engine.plugins.activity.ActivityPluginBinding
import kotlinx.coroutines.CompletableDeferred

/**
 * The system's sign-in session as a Custom Tab. The page's end at
 * `kanso://auth` arrives at [AuthCallbackActivity]; a tab the person closed
 * is noticed when the app comes back with no callback.
 */
class KansoAuthSessionPlugin :
    FlutterPlugin,
    ActivityAware,
    AuthSessionHostApi {
    private var activity: Activity? = null
    private var watcher: Application.ActivityLifecycleCallbacks? = null
    private val main = Handler(Looper.getMainLooper())

    override fun onAttachedToEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        AuthSessionHostApi.setUp(binding.binaryMessenger, this)
    }

    override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        AuthSessionHostApi.setUp(binding.binaryMessenger, null)
    }

    override fun onAttachedToActivity(binding: ActivityPluginBinding) {
        activity = binding.activity
    }

    override fun onDetachedFromActivityForConfigChanges() = onDetachedFromActivity()

    override fun onReattachedToActivityForConfigChanges(binding: ActivityPluginBinding) =
        onAttachedToActivity(binding)

    override fun onDetachedFromActivity() {
        stopWatching()
        activity = null
    }

    override suspend fun open(
        url: String,
        callbackScheme: String?,
    ): String? {
        // Only web pages: never a file, a script or another app's scheme.
        val page = Uri.parse(url)
        val scheme = page.scheme?.lowercase()
        if ((scheme != "http" && scheme != "https") || page.host.isNullOrEmpty()) {
            throw FlutterError("invalid_url", "Only http and https pages are opened.", null)
        }
        val host = activity ?: throw FlutterError("no_activity", "The app is not in front.", null)
        stopWatching()
        val session = AuthSessions.begin()
        watch(host, session)
        try {
            CustomTabsIntent.Builder().build().launchUrl(host, page)
        } catch (error: Exception) {
            stopWatching()
            AuthSessions.dismiss(session)
            throw FlutterError("unavailable", "No browser could open the sign-in page.", null)
        }
        try {
            return session.await()
        } finally {
            stopWatching()
        }
    }

    override fun close() {
        stopWatching()
        val context = activity?.applicationContext
        AuthSessions.dismiss()
        if (context != null) AuthSessions.bringAppToFront(context)
    }

    /**
     * Notices a tab the person closed: the host activity paused for the tab
     * and then resumed with the session still open and no callback. A short
     * wait lets a callback that is on its way arrive first.
     */
    private fun watch(
        host: Activity,
        session: CompletableDeferred<String?>,
    ) {
        var left = false
        val callbacks =
            object : Application.ActivityLifecycleCallbacks {
                override fun onActivityPaused(paused: Activity) {
                    if (paused === host) left = true
                }

                override fun onActivityResumed(resumed: Activity) {
                    if (resumed !== host || !left) return
                    main.postDelayed({ AuthSessions.dismiss(session) }, DISMISS_GRACE_MS)
                }

                override fun onActivityCreated(a: Activity, b: Bundle?) = Unit

                override fun onActivityStarted(a: Activity) = Unit

                override fun onActivityStopped(a: Activity) = Unit

                override fun onActivitySaveInstanceState(a: Activity, b: Bundle) = Unit

                override fun onActivityDestroyed(a: Activity) = Unit
            }
        host.application.registerActivityLifecycleCallbacks(callbacks)
        watcher = callbacks
    }

    private fun stopWatching() {
        val callbacks = watcher ?: return
        watcher = null
        main.removeCallbacksAndMessages(null)
        activity?.application?.unregisterActivityLifecycleCallbacks(callbacks)
    }

    private companion object {
        const val DISMISS_GRACE_MS = 600L
    }
}
