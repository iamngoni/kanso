package app.kanso.kanso_dictation

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class PcmConverterTest {
    @Test
    fun downmixAveragesChannels() {
        val out = PcmConverter.downmix(shortArrayOf(100, 300, -200, -400), 2)
        assertArrayEquals(shortArrayOf(200, -300), out)
    }

    @Test
    fun monoAt16kIsUnchanged() {
        val input = shortArrayOf(1, 2, 3, 4)
        assertArrayEquals(input, PcmConverter(16_000, 1).convert(input))
    }

    @Test
    fun bytesAreLittleEndian() {
        val bytes = PcmConverter.toBytes(shortArrayOf(0x0102, -2))
        assertArrayEquals(byteArrayOf(0x02, 0x01, 0xfe.toByte(), 0xff.toByte()), bytes)
    }

    @Test
    fun resamplingKeepsTheDuration() {
        // One second at 48 kHz becomes (about) one second at 16 kHz.
        val input = ShortArray(48_000) { (it % 100).toShort() }
        val out = PcmConverter(48_000, 1).convert(input)
        assertEquals(16_000.0, out.size.toDouble(), 2.0)
    }

    @Test
    fun resamplingIsTheSameWhateverThePieceSize() {
        val input = ShortArray(44_100) { ((it * 37) % 2000 - 1000).toShort() }
        val whole = PcmConverter(44_100, 1).convert(input)
        val pieces = PcmConverter(44_100, 1)
        val parts = ArrayList<Short>()
        var at = 0
        for (size in listOf(1, 7, 1000, 4096, 1)) {
            val end = minOf(input.size, at + size)
            parts += pieces.convert(input.copyOfRange(at, end)).toList()
            at = end
        }
        parts += pieces.convert(input.copyOfRange(at, input.size)).toList()
        assertArrayEquals(whole, parts.toShortArray())
    }

    @Test
    fun resamplingInterpolatesALine() {
        // A ramp stays a ramp: 32 kHz down to 16 kHz picks every other sample.
        val input = ShortArray(10) { (it * 10).toShort() }
        val out = PcmConverter(32_000, 1).convert(input)
        assertEquals(0, out[0].toInt())
        assertEquals(20, out[1].toInt())
        assertEquals(40, out[2].toInt())
    }

    @Test
    fun serverEngineNeedsPermissionToBeUsed() {
        val tiramisu = android.os.Build.VERSION_CODES.TIRAMISU
        val e = FileTranscriber.Engine.SERVER
        assertEquals("notOnDevice", FileTranscriber.refusal(tiramisu, true, e, false))
        assertNull(FileTranscriber.refusal(tiramisu, true, e, true))
    }

    @Test
    fun refusalsAreChecked_inOrder() {
        val tiramisu = android.os.Build.VERSION_CODES.TIRAMISU
        val onDevice = FileTranscriber.Engine.ON_DEVICE
        assertEquals("unsupported", FileTranscriber.refusal(tiramisu - 1, true, onDevice, true))
        assertEquals("denied", FileTranscriber.refusal(tiramisu, false, onDevice, true))
        assertEquals("unsupported", FileTranscriber.refusal(tiramisu, true, FileTranscriber.Engine.NONE, true))
        assertNull(FileTranscriber.refusal(tiramisu, true, onDevice, false))
    }

    @Test
    fun recognizerErrorsMapToCodes() {
        assertNull(FileTranscriber.errorFor(android.speech.SpeechRecognizer.ERROR_NO_MATCH))
        assertNull(FileTranscriber.errorFor(android.speech.SpeechRecognizer.ERROR_SPEECH_TIMEOUT))
        assertEquals("denied", FileTranscriber.errorFor(android.speech.SpeechRecognizer.ERROR_INSUFFICIENT_PERMISSIONS))
        assertEquals("failed", FileTranscriber.errorFor(android.speech.SpeechRecognizer.ERROR_NETWORK))
    }

    @Test
    fun timeoutGrowsWithTheAudio() {
        assertEquals(60_000L, FileTranscriber.timeoutMs(0))
        assertEquals(60_000L + 2 * 30_000, FileTranscriber.timeoutMs(30_000))
    }
}
