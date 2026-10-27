// Kanso — KansoPushService.kt
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

package app.kanso.kanso_push

import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage

/**
 * Receives Firebase Messaging's events. No Flutter engine is needed: a new
 * token is passed to Dart if one is attached and kept for the next
 * `register()` if not, and a message calls [KansoPush.onWake].
 */
class KansoPushService : FirebaseMessagingService() {
    override fun onNewToken(token: String) {
        KansoPushPlugin.tokenArrived(applicationContext, token)
    }

    override fun onMessageReceived(message: RemoteMessage) {
        // The message has no content to read; it only wakes the app.
        KansoPush.onWake?.invoke(applicationContext)
    }
}
