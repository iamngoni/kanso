package app.kanso.kanso_dictation

import android.speech.SpeechRecognizer
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class DictationMappingTest {
    @Test
    fun levelRunsFromQuietToLoudAndNeverLeavesTheRange() {
        assertEquals(0.0, DictationMapping.level(-100f), 0.0)
        assertEquals(0.0, DictationMapping.level(-2f), 0.0)
        assertEquals(0.5, DictationMapping.level(4f), 1e-6)
        assertEquals(1.0, DictationMapping.level(10f), 0.0)
        assertEquals(1.0, DictationMapping.level(60f), 0.0)
    }

    @Test
    fun silenceIsNoMatchAndSpeechTimeoutOnly() {
        assertTrue(DictationMapping.isSilence(SpeechRecognizer.ERROR_NO_MATCH))
        assertTrue(DictationMapping.isSilence(SpeechRecognizer.ERROR_SPEECH_TIMEOUT))
        assertFalse(DictationMapping.isSilence(SpeechRecognizer.ERROR_AUDIO))
    }

    @Test
    fun errorsMapToTheCodesDartKnows() {
        assertEquals(DictationErrorCode.NO_SPEECH, DictationMapping.errorCode(SpeechRecognizer.ERROR_NO_MATCH))
        assertEquals(DictationErrorCode.AUDIO, DictationMapping.errorCode(SpeechRecognizer.ERROR_AUDIO))
        assertEquals(
            DictationErrorCode.PERMISSION_DENIED,
            DictationMapping.errorCode(SpeechRecognizer.ERROR_INSUFFICIENT_PERMISSIONS),
        )
        assertEquals(DictationErrorCode.BUSY, DictationMapping.errorCode(SpeechRecognizer.ERROR_RECOGNIZER_BUSY))
        assertEquals(DictationErrorCode.NOT_AVAILABLE, DictationMapping.errorCode(SpeechRecognizer.ERROR_NETWORK))
        assertEquals(DictationErrorCode.UNKNOWN, DictationMapping.errorCode(9999))
    }

    @Test
    fun aClientErrorIsNotAFailureOfTheSession() {
        assertNull(DictationMapping.errorCode(SpeechRecognizer.ERROR_CLIENT))
    }
}
