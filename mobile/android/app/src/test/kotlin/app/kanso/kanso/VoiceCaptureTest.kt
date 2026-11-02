package app.kanso.kanso

import java.io.File
import java.nio.file.Files
import java.security.SecureRandom
import org.json.JSONObject
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue
import org.testng.annotations.AfterMethod
import org.testng.annotations.BeforeMethod
import org.testng.annotations.Test

class VoiceCaptureTest {
    private lateinit var root: File
    private lateinit var inbox: File

    @BeforeMethod
    fun setUp() {
        root = Files.createTempDirectory("kanso-voice").toFile()
        inbox = File(root, "inbox").also { it.mkdirs() }
    }

    @AfterMethod
    fun tearDown() {
        root.deleteRecursively()
    }

    private fun partial(id: String, bytes: Int = 10): File {
        val dir = File(inbox, "$id.partial").also { it.mkdir() }
        File(dir, VoiceCapture.AUDIO_NAME).writeBytes(ByteArray(bytes) { 1 })
        return dir
    }

    @Test
    fun toggleStopsWhenRecordingAndStartsOtherwise() {
        assertEquals(VoiceAction.STOP, VoiceCapture.toggle(true))
        assertEquals(VoiceAction.START, VoiceCapture.toggle(false))
    }

    @Test
    fun elapsedIsMinutesAndSeconds() {
        assertEquals("00:00", VoiceCapture.elapsed(0))
        assertEquals("00:14", VoiceCapture.elapsed(14_999))
        assertEquals("01:05", VoiceCapture.elapsed(65_000))
        assertEquals("30:00", VoiceCapture.elapsed(VoiceCapture.MAX_MS))
        assertEquals("1:00:01", VoiceCapture.elapsed(3_601_000))
        assertEquals("00:00", VoiceCapture.elapsed(-5))
    }

    @Test
    fun notificationTitleNamesTheTime() {
        assertEquals("Kanso is recording · 00:14", VoiceCapture.notificationTitle(14_200))
    }

    @Test
    fun idIs32LowercaseHex() {
        val a = VoiceCapture.newId(SecureRandom())
        val b = VoiceCapture.newId(SecureRandom())
        assertTrue(Regex("[0-9a-f]{32}").matches(a), a)
        assertTrue(a != b)
    }

    @Test
    fun limitsAreOneSecondAndThirtyMinutes() {
        assertFalse(VoiceCapture.keep(999))
        assertTrue(VoiceCapture.keep(1_000))
        assertFalse(VoiceCapture.atLimit(VoiceCapture.MAX_MS - 1))
        assertTrue(VoiceCapture.atLimit(VoiceCapture.MAX_MS))
    }

    @Test
    fun manifestHasTheDocumentedFields() {
        val json = JSONObject(VoiceCapture.manifest(1_000, 14_000, 15_000))
        assertEquals(1, json.getInt("version"))
        assertEquals("voice", json.getString("kind"))
        assertEquals(15_000L, json.getLong("createdAt"))
        assertEquals(1_000L, json.getLong("started_ms"))
        assertEquals(14_000L, json.getLong("duration_ms"))
        assertEquals("rec.m4a", json.getString("audio"))
        assertEquals(6, json.length())
    }

    @Test
    fun finishWritesTheManifestThenRenames() {
        val id = "a".repeat(32)
        partial(id)
        val done = VoiceCapture.finish(inbox, id, 1_000, 5_000, 6_000)
        assertNotNull(done)
        assertEquals(File(inbox, id), done)
        assertFalse(File(inbox, "$id.partial").exists())
        assertTrue(File(done, "rec.m4a").isFile)
        val json = JSONObject(File(done, "drop.json").readText(Charsets.UTF_8))
        assertEquals("voice", json.getString("kind"))
        assertEquals(5_000L, json.getLong("duration_ms"))
    }

    @Test
    fun finishDiscardsAShortRecording() {
        val id = "b".repeat(32)
        partial(id)
        assertNull(VoiceCapture.finish(inbox, id, 1_000, 400, 2_000))
        assertFalse(File(inbox, "$id.partial").exists())
        assertFalse(File(inbox, id).exists())
    }

    @Test
    fun finishDiscardsAMissingOrEmptyRecording() {
        val id = "c".repeat(32)
        partial(id, bytes = 0)
        assertNull(VoiceCapture.finish(inbox, id, 1_000, 5_000, 6_000))
        assertFalse(File(inbox, id).exists())
        assertFalse(File(inbox, "$id.partial").exists())
    }

    @Test
    fun tileIsActiveExactlyWhileRecording() {
        assertEquals(VoiceTileMode.ACTIVE, VoiceCapture.tileMode(true))
        assertEquals(VoiceTileMode.INACTIVE, VoiceCapture.tileMode(false))
    }

    @Test
    fun stateTellsListenersOnlyAboutChanges() {
        val seen = mutableListOf<Boolean>()
        val listener: (Boolean) -> Unit = { seen += it }
        VoiceState.listen(listener)
        try {
            VoiceState.set(true)
            VoiceState.set(true)
            VoiceState.set(false)
        } finally {
            VoiceState.unlisten(listener)
            VoiceState.set(false)
        }
        assertEquals(listOf(true, false), seen)
    }
}
