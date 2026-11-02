package app.kanso.kanso

import java.io.File
import java.io.IOException
import java.security.SecureRandom
import org.json.JSONObject

/** What a request to the voice recorder means, given whether it is recording. */
enum class VoiceAction { START, STOP }

/** How the Quick Settings tile looks. */
enum class VoiceTileMode { ACTIVE, INACTIVE }

/**
 * The rules of the voice recorder that need no Android API, so a plain JVM
 * test can run them (`VoiceCaptureService` and `VoiceTileService` use them).
 */
object VoiceCapture {
    /** A recording is stopped at this length, 30 minutes. */
    const val MAX_MS = 30L * 60 * 1000

    /** A recording shorter than this is thrown away, 1 second. */
    const val MIN_MS = 1_000L

    /** The audio file's name in the drop. */
    const val AUDIO_NAME = "rec.m4a"

    /** A tap or a second start request: stop when recording, start otherwise. */
    fun toggle(recording: Boolean): VoiceAction = if (recording) VoiceAction.STOP else VoiceAction.START

    /** Whether a recording of [durationMs] is long enough to keep. */
    fun keep(durationMs: Long): Boolean = durationMs >= MIN_MS

    /** Whether a recording of [elapsedMs] has reached the limit. */
    fun atLimit(elapsedMs: Long): Boolean = elapsedMs >= MAX_MS

    /** A fresh drop id: 32 lowercase hex characters, as the share writer makes. */
    fun newId(random: SecureRandom): String = DropWriter.newId(random)

    /** [elapsedMs] as `mm:ss` (`h:mm:ss` from an hour on); negative counts as zero. */
    fun elapsed(elapsedMs: Long): String {
        val total = maxOf(0L, elapsedMs) / 1000
        val hours = total / 3600
        val minutes = total % 3600 / 60
        val seconds = total % 60
        return if (hours > 0) {
            "%d:%02d:%02d".format(hours, minutes, seconds)
        } else {
            "%02d:%02d".format(minutes, seconds)
        }
    }

    /** The notification's title while recording: `Kanso is recording · 00:14`. */
    fun notificationTitle(elapsedMs: Long): String = "Kanso is recording · ${elapsed(elapsedMs)}"

    /** The drop's `drop.json` for a recording. */
    fun manifest(startedMs: Long, durationMs: Long, createdAtMs: Long): String =
        JSONObject()
            .put("version", 1)
            .put("kind", "voice")
            .put("createdAt", createdAtMs)
            .put("started_ms", startedMs)
            .put("duration_ms", durationMs)
            .put("audio", AUDIO_NAME)
            .toString()

    /** The tile's look: active exactly while recording. */
    fun tileMode(recording: Boolean): VoiceTileMode =
        if (recording) VoiceTileMode.ACTIVE else VoiceTileMode.INACTIVE

    /**
     * Finishes a voice drop the way [DropWriter] finishes a share: `drop.json`
     * last, then `<id>.partial` becomes `<id>`. A recording that is too short,
     * or whose audio is missing, is removed instead.
     *
     * @return the drop's folder, or `null` when the recording was discarded.
     * @throws IOException when the drop can't be finished (the partial is removed).
     */
    fun finish(inbox: File, id: String, startedMs: Long, durationMs: Long, nowMs: Long): File? {
        val partial = File(inbox, "$id.partial")
        val audio = File(partial, AUDIO_NAME)
        if (!keep(durationMs) || !audio.isFile || audio.length() == 0L) {
            partial.deleteRecursively()
            return null
        }
        try {
            File(partial, "drop.json").writeText(manifest(startedMs, durationMs, nowMs), Charsets.UTF_8)
            val done = File(inbox, id)
            if (!partial.renameTo(done)) {
                throw IOException("The drop can't be finished.")
            }
            return done
        } catch (error: Throwable) {
            partial.deleteRecursively()
            throw error
        }
    }
}

/**
 * Whether Kanso is recording right now, shared by the service, the tile and
 * the widget (one process). The service sets it; listeners (the tile) follow.
 */
object VoiceState {
    private val listeners = java.util.concurrent.CopyOnWriteArrayList<(Boolean) -> Unit>()

    @Volatile
    var recording: Boolean = false
        private set

    fun set(value: Boolean) {
        if (recording != value) {
            recording = value
            listeners.forEach { it(value) }
        }
    }

    fun listen(listener: (Boolean) -> Unit) {
        listeners += listener
    }

    fun unlisten(listener: (Boolean) -> Unit) {
        listeners -= listener
    }
}
