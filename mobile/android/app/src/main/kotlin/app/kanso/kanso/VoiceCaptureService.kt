package app.kanso.kanso

import android.Manifest
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.content.pm.ServiceInfo
import android.media.MediaRecorder
import android.net.Uri
import android.os.Build
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import android.os.SystemClock
import android.service.quicksettings.TileService
import java.io.File
import java.io.IOException
import java.security.SecureRandom

/**
 * Records a voice note in the background, from the Quick Settings tile, the
 * widget or the notification (`mobile/docs/capture.md`).
 *
 * It records AAC in an ADTS stream (readable even if killed), mono, 16 kHz, 32 kbps, into
 * `<filesDir>/kanso/inbox/<id>.partial/rec.aac`, and on stop finishes the
 * drop like a share ([VoiceCapture.finish]). It never opens the library or
 * calls the Rust core: the app transcribes the drop the next time it opens.
 *
 * It stops on the Stop action, on a second start request, and at 30 minutes.
 */
class VoiceCaptureService : Service() {
    private val handler = Handler(Looper.getMainLooper())
    private var recorder: MediaRecorder? = null
    private var inbox: File? = null
    private var id: String? = null
    private var startedWall = 0L
    private var startedTick = 0L
    private var foreground = false

    private val tick = object : Runnable {
        override fun run() {
            if (recorder == null) {
                return
            }
            val elapsed = SystemClock.elapsedRealtime() - startedTick
            if (VoiceCapture.atLimit(elapsed)) {
                stopRecording()
                return
            }
            notificationManager().notify(NOTIFICATION_ID, notification(elapsed))
            handler.postDelayed(this, 1000)
        }
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        // A service started with startForegroundService must call
        // startForeground soon, even when it has nothing to do.
        if (recorder == null && !canRecord(this)) {
            giveUp("Kanso needs the microphone. Tap to allow it.")
            return START_NOT_STICKY
        }
        if (!enterForeground(if (recorder != null) SystemClock.elapsedRealtime() - startedTick else 0)) {
            giveUp("Kanso needs the microphone. Tap to allow it.")
            return START_NOT_STICKY
        }
        val action = if (intent?.action == ACTION_STOP) {
            VoiceAction.STOP
        } else {
            VoiceCapture.toggle(recorder != null)
        }
        when (action) {
            VoiceAction.STOP -> stopRecording()
            VoiceAction.START -> startRecording()
        }
        return START_NOT_STICKY
    }

    override fun onDestroy() {
        if (recorder != null) {
            stopRecording()
        }
        VoiceState.set(false)
        super.onDestroy()
    }

    /** False when Android refuses the microphone foreground service (Android 14 and later). */
    private fun enterForeground(elapsed: Long): Boolean {
        if (foreground) {
            return true
        }
        ensureChannel()
        return try {
            startForeground(
                NOTIFICATION_ID,
                notification(elapsed),
                ServiceInfo.FOREGROUND_SERVICE_TYPE_MICROPHONE,
            )
            foreground = true
            true
        } catch (_: RuntimeException) {
            false
        }
    }

    private fun startRecording() {
        if (!canRecord(this)) {
            giveUp("Kanso needs the microphone. Tap to allow it.")
            return
        }
        val inbox = inboxDir(this)
        sweep(this)
        val id = VoiceCapture.newId(SecureRandom())
        val partial = File(inbox, "$id.partial")
        if (!(inbox.isDirectory || inbox.mkdirs()) || !partial.mkdir()) {
            giveUp("Kanso couldn't start recording.")
            return
        }
        val media = newRecorder()
        try {
            media.setAudioSource(MediaRecorder.AudioSource.MIC)
            media.setOutputFormat(MediaRecorder.OutputFormat.AAC_ADTS)
            media.setAudioEncoder(MediaRecorder.AudioEncoder.AAC)
            media.setAudioChannels(1)
            media.setAudioSamplingRate(VoiceCapture.SAMPLE_RATE)
            media.setAudioEncodingBitRate(VoiceCapture.BIT_RATE)
            media.setMaxDuration(VoiceCapture.MAX_MS.toInt())
            media.setOnInfoListener { _, what, _ ->
                if (what == MediaRecorder.MEDIA_RECORDER_INFO_MAX_DURATION_REACHED) {
                    stopRecording()
                }
            }
            VoiceCapture.markStarted(partial, System.currentTimeMillis())
            media.setOutputFile(File(partial, VoiceCapture.AUDIO_NAME).path)
            media.prepare()
            media.start()
        } catch (_: Exception) {
            media.release()
            partial.deleteRecursively()
            giveUp("Kanso couldn't start recording.")
            return
        }
        this.inbox = inbox
        this.id = id
        this.recorder = media
        startedWall = System.currentTimeMillis()
        startedTick = SystemClock.elapsedRealtime()
        VoiceState.set(true)
        refreshTile(this)
        notificationManager().notify(NOTIFICATION_ID, notification(0))
        handler.postDelayed(tick, 1000)
    }

    private fun stopRecording() {
        handler.removeCallbacks(tick)
        val media = recorder
        val inbox = inbox
        val id = id
        recorder = null
        this.inbox = null
        this.id = null
        VoiceState.set(false)
        refreshTile(this)
        if (media != null) {
            // Elapsed time never depends on stop() succeeding: ADTS keeps what was written.
            val duration = if (startedTick > 0) SystemClock.elapsedRealtime() - startedTick else System.currentTimeMillis() - startedWall
            try {
                media.setOnInfoListener(null)
                media.stop()
            } catch (_: RuntimeException) {
                // stop() throws when little audio was written; the file is still judged below.
            } finally {
                media.release()
            }
            try {
                if (id != null) {
                    VoiceCapture.finish(
                        inbox ?: inboxDir(this), id, startedWall, duration, System.currentTimeMillis(),
                    )
                }
            } catch (_: IOException) {
                // The partial is removed; nothing more to do.
            }
        }
        stopForeground(STOP_FOREGROUND_REMOVE)
        foreground = false
        stopSelf()
    }

    /** Nothing was recorded: say why, with a tap that opens the app's voice note. */
    private fun giveUp(message: String) {
        stopForeground(STOP_FOREGROUND_REMOVE)
        foreground = false
        val open = PendingIntent.getActivity(
            this, 1, voiceLink(this), PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
        val note = Notification.Builder(this, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_mic)
            .setContentTitle(message)
            .setContentIntent(open)
            .setAutoCancel(true)
            .build()
        notificationManager().notify(FAILURE_ID, note)
        stopSelf()
    }

    private fun notification(elapsed: Long): Notification {
        val launch = packageManager.getLaunchIntentForPackage(packageName) ?: voiceLink(this)
        val open = PendingIntent.getActivity(
            this, 0, launch,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
        val stop = PendingIntent.getService(
            this, 2, Intent(this, VoiceCaptureService::class.java).setAction(ACTION_STOP),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
        return Notification.Builder(this, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_mic)
            .setContentTitle(VoiceCapture.notificationTitle(elapsed))
            .setContentIntent(open)
            .addAction(Notification.Action.Builder(null, "Stop", stop).build())
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .setCategory(Notification.CATEGORY_SERVICE)
            .build()
    }

    private fun ensureChannel() {
        val channel = NotificationChannel(CHANNEL_ID, "Voice notes", NotificationManager.IMPORTANCE_LOW)
        channel.description = "Shown while Kanso is recording a voice note."
        notificationManager().createNotificationChannel(channel)
    }

    private fun notificationManager() = getSystemService(NotificationManager::class.java)

    @Suppress("DEPRECATION")
    private fun newRecorder(): MediaRecorder =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) MediaRecorder(this) else MediaRecorder()

    companion object {
        const val ACTION_STOP = "app.kanso.kanso.VOICE_STOP"
        const val VOICE_LINK = "kanso://open/new?voice=1"
        private const val CHANNEL_ID = "voice_capture"
        private const val NOTIFICATION_ID = 0x564f
        private const val FAILURE_ID = 0x5646

        /** The intent that opens the app's voice note page, where the permission flow runs. */
        fun voiceLink(context: Context): Intent =
            Intent(Intent.ACTION_VIEW, Uri.parse(VOICE_LINK))
                .setClassName(context, MainActivity::class.java.name)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)

        fun inboxDir(context: Context): File = File(File(context.filesDir, "kanso"), "inbox")

        /**
         * Turns stale partials (a killed recording) into real drops. Call it
         * when the app or the service starts.
         */
        fun sweep(context: Context) {
            try {
                VoiceCapture.recoverStale(inboxDir(context), VoiceState.recording, System.currentTimeMillis())
            } catch (_: Exception) {
                // Never block start-up.
            }
        }

        /** Whether the microphone permission is granted. */
        fun canRecord(context: Context): Boolean =
            context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED

        /** The intent that starts (or, while recording, stops) the service. */
        fun toggleIntent(context: Context): Intent = Intent(context, VoiceCaptureService::class.java)

        /**
         * Starts or toggles recording from a visible action (a tile or widget
         * tap). Returns false when the service can't be started (no
         * microphone permission, or Android refuses the start); the caller
         * then opens [voiceLink] so the app's own permission flow runs.
         */
        fun start(context: Context): Boolean {
            if (!VoiceState.recording && !canRecord(context)) {
                return false
            }
            return try {
                context.startForegroundService(toggleIntent(context))
                true
            } catch (_: RuntimeException) {
                // ForegroundServiceStartNotAllowedException (Android 12 and
                // later) or a background-start refusal.
                false
            }
        }

        /** Asks the tile to read [VoiceState] again. */
        fun refreshTile(context: Context) {
            try {
                TileService.requestListeningState(context, ComponentName(context, VoiceTileService::class.java))
            } catch (_: RuntimeException) {
                // No tile added; nothing to refresh.
            }
        }
    }
}
