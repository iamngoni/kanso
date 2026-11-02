package app.kanso.kanso_dictation

import android.Manifest
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.media.AudioFormat
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.os.ParcelFileDescriptor
import android.speech.RecognitionListener
import android.speech.RecognizerIntent
import android.speech.SpeechRecognizer
import java.io.File
import java.io.IOException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withContext
import kotlin.coroutines.resume

/** The words of a file, or the reason there are none. */
object FileTranscriber {
    /** Error codes the Dart side knows (`FileTranscription.error`). */
    const val UNSUPPORTED = "unsupported"
    const val NOT_ON_DEVICE = "notOnDevice"
    const val DENIED = "denied"
    const val UNREADABLE = "unreadable"
    const val FAILED = "failed"

    /** Which recognizer a device offers a file: on the device, the system's server one, or none. */
    enum class Engine { ON_DEVICE, SERVER, NONE }

    /**
     * The error to report before any work is done, or `null` to go on.
     * Pure: [engine] is what the device has, [allowServer] what the person allows.
     */
    fun refusal(sdk: Int, permissionGranted: Boolean, engine: Engine, allowServer: Boolean): String? =
        when {
            sdk < Build.VERSION_CODES.TIRAMISU -> UNSUPPORTED
            !permissionGranted -> DENIED
            engine == Engine.NONE -> UNSUPPORTED
            engine == Engine.SERVER && !allowServer -> NOT_ON_DEVICE
            else -> null
        }

    /** What an `SpeechRecognizer.ERROR_*` code means for a file: empty text, or an error. */
    fun errorFor(code: Int): String? =
        when (code) {
            SpeechRecognizer.ERROR_NO_MATCH, SpeechRecognizer.ERROR_SPEECH_TIMEOUT -> null
            SpeechRecognizer.ERROR_INSUFFICIENT_PERMISSIONS -> DENIED
            else -> FAILED
        }

    /** The time the recognizer is given for audio of [audioMs]. */
    fun timeoutMs(audioMs: Long): Long = 60_000L + audioMs * 2

    suspend fun transcribe(
        context: Context,
        path: String,
        localeTag: String?,
        allowServer: Boolean,
    ): FileTranscription {
        val granted = context.checkSelfPermission(Manifest.permission.RECORD_AUDIO) ==
            PackageManager.PERMISSION_GRANTED
        val engine = withContext(Dispatchers.Main) { engine(context) }
        refusal(Build.VERSION.SDK_INT, granted, engine, allowServer)?.let { return failure(it) }
        val source = File(path)
        if (!source.isFile) {
            return failure(UNREADABLE)
        }
        val pcm = File.createTempFile("kanso-transcribe", ".pcm", context.cacheDir)
        try {
            val decoded = try {
                withContext(Dispatchers.IO) { AudioFileDecoder.decode(source, pcm) }
            } catch (_: IOException) {
                return failure(UNREADABLE)
            }
            if (pcm.length() == 0L) {
                return FileTranscription(text = "", onDevice = engine == Engine.ON_DEVICE)
            }
            return withContext(Dispatchers.Main) {
                recognize(context, pcm, decoded.durationMs, localeTag, engine)
            }
        } finally {
            pcm.delete()
        }
    }

    private fun failure(code: String) = FileTranscription(text = "", onDevice = false, error = code)

    private fun engine(context: Context): Engine =
        when {
            Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU &&
                SpeechRecognizer.isOnDeviceRecognitionAvailable(context) -> Engine.ON_DEVICE
            SpeechRecognizer.isRecognitionAvailable(context) -> Engine.SERVER
            else -> Engine.NONE
        }

    private suspend fun recognize(
        context: Context,
        pcm: File,
        audioMs: Long,
        localeTag: String?,
        engine: Engine,
    ): FileTranscription =
        suspendCancellableCoroutine { continuation ->
            val handler = Handler(Looper.getMainLooper())
            val onDevice = engine == Engine.ON_DEVICE
            val recognizer =
                if (onDevice) {
                    SpeechRecognizer.createOnDeviceSpeechRecognizer(context)
                } else {
                    SpeechRecognizer.createSpeechRecognizer(context)
                }
            val descriptor = try {
                ParcelFileDescriptor.open(pcm, ParcelFileDescriptor.MODE_READ_ONLY)
            } catch (_: IOException) {
                recognizer.destroy()
                continuation.resume(failure(UNREADABLE))
                return@suspendCancellableCoroutine
            }
            var done = false
            fun finish(result: FileTranscription) {
                if (done) return
                done = true
                handler.removeCallbacksAndMessages(null)
                recognizer.destroy()
                try {
                    descriptor.close()
                } catch (_: IOException) {
                }
                if (continuation.isActive) continuation.resume(result)
            }
            val heard = StringBuilder()
            recognizer.setRecognitionListener(
                object : RecognitionListener {
                    override fun onReadyForSpeech(params: Bundle?) {}
                    override fun onBeginningOfSpeech() {}
                    override fun onRmsChanged(rmsdB: Float) {}
                    override fun onBufferReceived(buffer: ByteArray?) {}
                    override fun onEndOfSpeech() {}
                    override fun onEvent(eventType: Int, params: Bundle?) {}
                    override fun onPartialResults(partialResults: Bundle?) {}

                    override fun onError(error: Int) {
                        val code = errorFor(error)
                        finish(
                            if (code == null) {
                                FileTranscription(text = heard.toString().trim(), onDevice = onDevice)
                            } else {
                                FileTranscription(text = "", onDevice = onDevice, error = code)
                            },
                        )
                    }

                    override fun onResults(results: Bundle?) {
                        val text = results
                            ?.getStringArrayList(SpeechRecognizer.RESULTS_RECOGNITION)
                            ?.firstOrNull()
                            .orEmpty()
                        if (text.isNotEmpty()) {
                            if (heard.isNotEmpty()) heard.append(' ')
                            heard.append(text)
                        }
                        finish(FileTranscription(text = heard.toString().trim(), onDevice = onDevice))
                    }
                },
            )
            val intent = Intent(RecognizerIntent.ACTION_RECOGNIZE_SPEECH).apply {
                putExtra(RecognizerIntent.EXTRA_LANGUAGE_MODEL, RecognizerIntent.LANGUAGE_MODEL_FREE_FORM)
                putExtra(RecognizerIntent.EXTRA_CALLING_PACKAGE, context.packageName)
                putExtra(RecognizerIntent.EXTRA_PREFER_OFFLINE, onDevice)
                localeTag?.let { putExtra(RecognizerIntent.EXTRA_LANGUAGE, it) }
                putExtra(RecognizerIntent.EXTRA_AUDIO_SOURCE, descriptor)
                putExtra(RecognizerIntent.EXTRA_AUDIO_SOURCE_CHANNEL_COUNT, 1)
                putExtra(RecognizerIntent.EXTRA_AUDIO_SOURCE_SAMPLING_RATE, PcmConverter.TARGET_RATE)
                putExtra(RecognizerIntent.EXTRA_AUDIO_SOURCE_ENCODING, AudioFormat.ENCODING_PCM_16BIT)
            }
            continuation.invokeOnCancellation {
                handler.post {
                    if (!done) {
                        done = true
                        recognizer.destroy()
                        try {
                            descriptor.close()
                        } catch (_: IOException) {
                        }
                    }
                }
            }
            handler.postDelayed({ finish(failure(FAILED)) }, timeoutMs(audioMs))
            recognizer.startListening(intent)
        }
}
