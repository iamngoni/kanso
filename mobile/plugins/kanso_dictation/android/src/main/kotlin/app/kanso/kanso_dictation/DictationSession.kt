package app.kanso.kanso_dictation

import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.speech.RecognitionListener
import android.speech.RecognizerIntent
import android.speech.SpeechRecognizer

/**
 * One run of the recognizer. It runs on the main thread, as `SpeechRecognizer`
 * requires, and reports through [emit]; the last thing it reports, once, is
 * an `ended` event.
 *
 * A `SpeechRecognizer` stops by itself after a phrase, so while the person has
 * not stopped it, a phrase's final result starts the next listen. Silence
 * after something was said, or a long silence, ends the session.
 */
class DictationSession(
    private val context: Context,
    private val localeTag: String?,
    private val emit: (DictationEvent) -> Unit,
) : RecognitionListener {
    private val handler = Handler(Looper.getMainLooper())
    private var recognizer: SpeechRecognizer? = null
    private var stopping = false
    private var heardSomething = false
    private var lastLevelAt = 0L
    private var listeningReported = false

    var finished = false
        private set

    /** Whether the recognizer of the device itself is used, not the system's. */
    val onDevice: Boolean =
        Build.VERSION.SDK_INT >= Build.VERSION_CODES.S &&
            SpeechRecognizer.isOnDeviceRecognitionAvailable(context)

    /** Starts listening; false when no recognizer can be made. */
    fun begin(): Boolean {
        val made =
            if (onDevice) {
                SpeechRecognizer.createOnDeviceSpeechRecognizer(context)
            } else if (SpeechRecognizer.isRecognitionAvailable(context)) {
                SpeechRecognizer.createSpeechRecognizer(context)
            } else {
                null
            } ?: return false
        made.setRecognitionListener(this)
        recognizer = made
        listen()
        return true
    }

    /** Stops listening and lets the recognizer finish what it heard. */
    fun stop() {
        if (finished || stopping) return
        stopping = true
        recognizer?.stopListening()
        // The recognizer may never answer a stop it was given too early.
        handler.postDelayed({ end() }, FINISH_TIMEOUT_MS)
    }

    /** Stops at once, dropping what the recognizer had not finished. */
    fun cancel() {
        if (finished) return
        recognizer?.cancel()
        end()
    }

    private fun listen() {
        val intent =
            Intent(RecognizerIntent.ACTION_RECOGNIZE_SPEECH).apply {
                putExtra(
                    RecognizerIntent.EXTRA_LANGUAGE_MODEL,
                    RecognizerIntent.LANGUAGE_MODEL_FREE_FORM,
                )
                putExtra(RecognizerIntent.EXTRA_PARTIAL_RESULTS, true)
                putExtra(RecognizerIntent.EXTRA_PREFER_OFFLINE, true)
                putExtra(RecognizerIntent.EXTRA_CALLING_PACKAGE, context.packageName)
                localeTag?.let { putExtra(RecognizerIntent.EXTRA_LANGUAGE, it) }
            }
        recognizer?.startListening(intent)
    }

    private fun end() {
        if (finished) return
        finished = true
        handler.removeCallbacksAndMessages(null)
        recognizer?.destroy()
        recognizer = null
        emit(DictationEvent(kind = DictationEventKind.ENDED))
    }

    private fun text(results: Bundle?): String? =
        results
            ?.getStringArrayList(SpeechRecognizer.RESULTS_RECOGNITION)
            ?.firstOrNull()
            ?.takeIf { it.isNotEmpty() }

    override fun onReadyForSpeech(params: Bundle?) {
        if (!listeningReported) {
            listeningReported = true
            emit(DictationEvent(kind = DictationEventKind.LISTENING))
        }
    }

    override fun onBeginningOfSpeech() {}

    override fun onRmsChanged(rmsdB: Float) {
        val now = System.currentTimeMillis()
        if (finished || now - lastLevelAt < LEVEL_INTERVAL_MS) return
        lastLevelAt = now
        emit(DictationEvent(kind = DictationEventKind.LEVEL, level = DictationMapping.level(rmsdB)))
    }

    override fun onBufferReceived(buffer: ByteArray?) {}

    override fun onEndOfSpeech() {}

    override fun onPartialResults(partialResults: Bundle?) {
        if (finished) return
        text(partialResults)?.let {
            heardSomething = true
            emit(DictationEvent(kind = DictationEventKind.PARTIAL, text = it))
        }
    }

    override fun onResults(results: Bundle?) {
        if (finished) return
        text(results)?.let {
            heardSomething = true
            emit(DictationEvent(kind = DictationEventKind.FINAL_RESULT, text = it))
        }
        if (stopping) {
            end()
        } else {
            // The next phrase, once the recognizer is done with this one.
            handler.post { if (!finished && !stopping) listen() }
        }
    }

    override fun onError(error: Int) {
        if (finished) return
        // Silence after the person spoke, or after they stopped, is the end.
        if (stopping || (DictationMapping.isSilence(error) && heardSomething)) {
            end()
            return
        }
        DictationMapping.errorCode(error)?.let {
            emit(DictationEvent(kind = DictationEventKind.ERROR, error = it, message = "SpeechRecognizer error $error"))
        }
        end()
    }

    override fun onEvent(eventType: Int, params: Bundle?) {}

    private companion object {
        const val FINISH_TIMEOUT_MS = 3000L
        const val LEVEL_INTERVAL_MS = 66L
    }
}
