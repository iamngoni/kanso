// Kanso — KansoPush.kt
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

package app.kanso.kanso_push

import android.content.Context

/** The hooks the app sets so a push can start its work. */
object KansoPush {
    /**
     * Called, on a Firebase Messaging thread, for each data message: a push
     * carries nothing and only says "sync". The app sets it in its
     * `Application.onCreate`; the service starts the process first, so it is
     * set by the time a message arrives. Left null, a message does nothing.
     */
    @JvmStatic
    @Volatile
    var onWake: ((Context) -> Unit)? = null
}
