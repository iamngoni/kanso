package app.kanso.kanso_dictation

import android.speech.SpeechRecognizer

/** What the recognizer's numbers mean to Dart. Pure, so a unit test can hold it. */
object DictationMapping {
    /** The loudest and the quietest voice the recognizer reports, in dB. */
    private const val QUIET_DB = -2f
    private const val LOUD_DB = 10f

    /** 0 to 1 for a recognizer's `onRmsChanged` value. */
    fun level(rmsDb: Float): Double =
        ((rmsDb - QUIET_DB) / (LOUD_DB - QUIET_DB)).coerceIn(0f, 1f).toDouble()

    /**
     * What a `SpeechRecognizer` error is to Dart; null for one that is not a
     * failure of the session (a client error after a cancel).
     */
    fun errorCode(error: Int): DictationErrorCode? =
        when (error) {
            SpeechRecognizer.ERROR_NO_MATCH,
            SpeechRecognizer.ERROR_SPEECH_TIMEOUT -> DictationErrorCode.NO_SPEECH
            SpeechRecognizer.ERROR_AUDIO -> DictationErrorCode.AUDIO
            SpeechRecognizer.ERROR_INSUFFICIENT_PERMISSIONS -> DictationErrorCode.PERMISSION_DENIED
            SpeechRecognizer.ERROR_RECOGNIZER_BUSY -> DictationErrorCode.BUSY
            SpeechRecognizer.ERROR_NETWORK,
            SpeechRecognizer.ERROR_NETWORK_TIMEOUT,
            SpeechRecognizer.ERROR_SERVER,
            SpeechRecognizer.ERROR_LANGUAGE_NOT_SUPPORTED,
            SpeechRecognizer.ERROR_LANGUAGE_UNAVAILABLE,
            SpeechRecognizer.ERROR_SERVER_DISCONNECTED,
            SpeechRecognizer.ERROR_TOO_MANY_REQUESTS -> DictationErrorCode.NOT_AVAILABLE
            SpeechRecognizer.ERROR_CLIENT -> null
            else -> DictationErrorCode.UNKNOWN
        }

    /** Whether hearing nothing is the normal end of a dictation, not a failure. */
    fun isSilence(error: Int): Boolean =
        error == SpeechRecognizer.ERROR_NO_MATCH || error == SpeechRecognizer.ERROR_SPEECH_TIMEOUT
}
