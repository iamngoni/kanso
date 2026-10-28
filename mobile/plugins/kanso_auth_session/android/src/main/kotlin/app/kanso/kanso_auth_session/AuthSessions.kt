package app.kanso.kanso_auth_session

import android.content.Context
import android.content.Intent
import android.net.Uri
import kotlinx.coroutines.CompletableDeferred

/**
 * The one open sign-in session of the process: what ends it, once.
 *
 * Main thread only, except [deliver] which the callback activity calls on the
 * main thread too.
 */
object AuthSessions {
    private var pending: CompletableDeferred<String?>? = null

    /** Starts a session; an earlier one ends as dismissed. */
    fun begin(): CompletableDeferred<String?> {
        pending?.complete(null)
        return CompletableDeferred<String?>().also { pending = it }
    }

    /** The session in progress, if any. */
    fun current(): CompletableDeferred<String?>? = pending

    /** Ends the session as dismissed (the tab was closed, or [KansoAuthSessionPlugin.close]). */
    fun dismiss() {
        pending?.complete(null)
        pending = null
    }

    /** Ends the session if [session] is still the open one. */
    fun dismiss(session: CompletableDeferred<String?>) {
        if (pending === session) dismiss()
    }

    /**
     * The page ended at [url]. Only a `kanso://auth` URL is taken, and only
     * while a session is waiting; anything else (another app firing the
     * exported activity) is dropped.
     */
    fun deliver(url: Uri?): Boolean {
        val waiting = pending ?: return false
        if (url == null || url.scheme != "kanso" || url.host != "auth") return false
        pending = null
        waiting.complete(url.toString())
        return true
    }

    /** Brings the app's task back in front, which closes a Custom Tab above it. */
    fun bringAppToFront(context: Context) {
        val launch = context.packageManager.getLaunchIntentForPackage(context.packageName) ?: return
        launch.addFlags(
            Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP,
        )
        context.startActivity(launch)
    }
}
