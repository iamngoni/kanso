package app.kanso.kanso

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder

/**
 * Keeps the app's process alive and its microphone open while the Record
 * screen is recording and the app goes to the background. The recorder is
 * the Rust core's; this service only holds the foreground state and the
 * ongoing notification. [KansoRecordingSession] starts it when the
 * recording prepares and stops it on release.
 */
class KansoRecordingService : Service() {
    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        ensureChannel(this)
        val notification = notification(this)
        val stopping = intent?.action == ACTION_STOP
        try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                startForeground(
                    NOTIFICATION_ID,
                    notification,
                    ServiceInfo.FOREGROUND_SERVICE_TYPE_MICROPHONE,
                )
            } else {
                startForeground(NOTIFICATION_ID, notification)
            }
        } catch (_: RuntimeException) {
            // Android refuses the microphone service (no permission, or not
            // allowed to start from here): do not leak the service.
            stopSelf()
            return START_NOT_STICKY
        }
        // A stop that arrived before the service reached the foreground: it
        // got there just above (Android crashes the app if a service started
        // with startForegroundService never calls startForeground), and now
        // it can go.
        if (stopping) {
            stopForegroundCompat()
            stopSelf()
        }
        return START_NOT_STICKY
    }

    /**
     * The app's task was swiped away mid-recording. The recorder is the Rust
     * core's and the process is about to be killed: what reached the disk is
     * kept in its session folder, and the next launch offers it for Recover
     * (the same way as after a crash). Nothing is deleted here; the service
     * just lets go of the microphone and its notification.
     */
    override fun onTaskRemoved(rootIntent: Intent?) {
        requested = false
        stopForegroundCompat()
        stopSelf()
        super.onTaskRemoved(rootIntent)
    }

    override fun onDestroy() {
        requested = false
        super.onDestroy()
    }

    private fun stopForegroundCompat() {
        try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
                stopForeground(STOP_FOREGROUND_REMOVE)
            } else {
                @Suppress("DEPRECATION")
                stopForeground(true)
            }
        } catch (_: RuntimeException) {
            // Not in the foreground: nothing to take down.
        }
    }

    companion object {
        private const val CHANNEL_ID = "kanso_recording"
        private const val NOTIFICATION_ID = 4802
        private const val ACTION_STOP = "app.kanso.kanso.action.STOP_RECORDING"

        /** Whether a start was asked for and no stop has followed yet. */
        @Volatile
        private var requested = false

        fun start(context: Context) {
            val app = context.applicationContext
            val intent = Intent(app, KansoRecordingService::class.java)
            requested = true
            try {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                    app.startForegroundService(intent)
                } else {
                    app.startService(intent)
                }
            } catch (_: RuntimeException) {
                // Recording still works in the foreground.
            }
        }

        fun stop(context: Context) {
            if (!requested) return
            requested = false
            val app = context.applicationContext
            // Not stopService: a service started but not yet in the
            // foreground would be stopped without calling startForeground,
            // which crashes the app on Android 8+. The stop is an intent in
            // the same queue as the start, so the service always reaches the
            // foreground first, then goes.
            val intent = Intent(app, KansoRecordingService::class.java).setAction(ACTION_STOP)
            try {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                    app.startForegroundService(intent)
                } else {
                    app.startService(intent)
                }
            } catch (_: RuntimeException) {
                // Not allowed to start it from here: nothing more to do
                // than ask it to stop.
                app.stopService(Intent(app, KansoRecordingService::class.java))
            }
        }

        private fun ensureChannel(context: Context) {
            if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
            val manager = context.getSystemService(NotificationManager::class.java)
            if (manager.getNotificationChannel(CHANNEL_ID) != null) return
            val channel = NotificationChannel(
                CHANNEL_ID,
                context.getString(R.string.recording_channel_name),
                NotificationManager.IMPORTANCE_LOW,
            )
            channel.setSound(null, null)
            channel.enableVibration(false)
            channel.setShowBadge(false)
            manager.createNotificationChannel(channel)
        }

        private fun notification(context: Context): Notification {
            val open = Intent(context, MainActivity::class.java)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP)
            val tap = PendingIntent.getActivity(
                context,
                0,
                open,
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
            )
            val builder = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                Notification.Builder(context, CHANNEL_ID)
            } else {
                @Suppress("DEPRECATION")
                Notification.Builder(context)
            }
            return builder
                .setSmallIcon(R.drawable.ic_mic)
                .setContentTitle(context.getString(R.string.recording_notification_title))
                .setContentText(context.getString(R.string.recording_notification_text))
                .setContentIntent(tap)
                .setOngoing(true)
                .setOnlyAlertOnce(true)
                .setCategory(Notification.CATEGORY_SERVICE)
                .build()
        }
    }
}
