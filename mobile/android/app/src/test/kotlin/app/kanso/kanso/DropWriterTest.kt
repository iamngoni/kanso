package app.kanso.kanso

import java.io.ByteArrayInputStream
import java.io.File
import java.io.IOException
import java.io.InputStream
import java.nio.file.Files
import java.security.SecureRandom
import org.json.JSONObject
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue
import org.testng.annotations.AfterMethod
import org.testng.annotations.BeforeMethod
import org.testng.annotations.Test

class DropWriterTest {
    private lateinit var root: File
    private lateinit var inbox: File
    private lateinit var writer: DropWriter

    @BeforeMethod
    fun setUp() {
        root = Files.createTempDirectory("kanso-drop").toFile()
        inbox = File(root, "inbox")
        writer = DropWriter(inbox, SecureRandom()) { 1_793_000_000_000L }
    }

    @AfterMethod
    fun tearDown() {
        root.deleteRecursively()
    }

    private fun file(name: String?, type: String?, bytes: ByteArray = byteArrayOf(1, 2, 3), size: Long? = bytes.size.toLong()) =
        DropSource(name, type, size) { ByteArrayInputStream(bytes) }

    private fun json(id: String) = JSONObject(File(File(inbox, id), "drop.json").readText())

    @Test
    fun idIs32LowercaseHex() {
        repeat(20) {
            assertTrue(Regex("[0-9a-f]{32}").matches(DropWriter.newId(SecureRandom())))
        }
    }

    @Test
    fun textDropHasTheDocumentedShapeAndNoPartialFolderRemains() {
        val result = writer.write("A title", "https://example.com/a", emptyList())
        val id = result.id!!
        val drop = json(id)
        assertEquals(1, drop.getInt("version"))
        assertEquals(1_793_000_000_000L, drop.getLong("createdAt"))
        assertEquals("A title", drop.getString("subject"))
        assertEquals("https://example.com/a", drop.getString("url"))
        assertTrue(!drop.has("files"))
        assertEquals(listOf(id), inbox.list()!!.toList())
    }

    @Test
    fun filesAreNumberedAndDescribed() {
        val result = writer.write(null, null, listOf(
            file("IMG_0042.PNG", "image/png"),
            file("Quarterly report.pdf", "application/pdf"),
            file(null, "image/jpeg"),
        ))
        val files = json(result.id!!).getJSONArray("files")
        assertEquals("0.png", files.getJSONObject(0).getString("path"))
        assertEquals("IMG_0042.PNG", files.getJSONObject(0).getString("name"))
        assertEquals("1.pdf", files.getJSONObject(1).getString("path"))
        assertEquals("2.jpg", files.getJSONObject(2).getString("path"))
        assertTrue(File(File(inbox, result.id!!), "2.jpg").exists())
    }

    @Test
    fun atMostTwentyFilesAreKept() {
        val result = writer.write(null, null, (1..25).map { file("f$it.png", "image/png") })
        assertEquals(20, json(result.id!!).getJSONArray("files").length())
        assertEquals(5, result.problems.size)
    }

    @Test
    fun aFileKnownToBeTooLargeIsSkippedWithoutBeingRead() {
        var opened = false
        val big = DropSource("big.pdf", "application/pdf", DropWriter.MAX_FILE_BYTES + 1) {
            opened = true
            ByteArrayInputStream(ByteArray(1))
        }
        val result = writer.write(null, "note", listOf(big))
        assertTrue(!opened)
        assertEquals(1, result.problems.size)
        assertTrue(!json(result.id!!).has("files"))
    }

    @Test
    fun aFileThatTurnsOutTooLargeIsRemoved() {
        val endless = object : InputStream() {
            override fun read() = 1
            override fun read(b: ByteArray, off: Int, len: Int) = len
        }
        val result = writer.write(null, "note", listOf(DropSource("x.png", "image/png", null) { endless }))
        assertEquals(1, result.problems.size)
        assertEquals(listOf("drop.json"), File(inbox, result.id!!).list()!!.toList())
    }

    @Test
    fun anUnreadableFileIsReportedAndTheRestIsKept() {
        val broken = DropSource("a.png", "image/png", null) { throw IOException("gone") }
        val result = writer.write(null, null, listOf(broken, file("b.png", "image/png")))
        assertEquals(1, result.problems.size)
        val files = json(result.id!!).getJSONArray("files")
        assertEquals(1, files.length())
        assertEquals("0.png", files.getJSONObject(0).getString("path"))
    }

    @Test
    fun nothingToKeepMakesNoDrop() {
        val result = writer.write("s", "  ", listOf(DropSource("a.png", null, null) { null }))
        assertNull(result.id)
        assertEquals(emptyList<String>(), inbox.list()!!.toList())
    }

    @Test
    fun extensionsComeFromTheNameThenTheType() {
        assertEquals("png", DropWriter.extensionOf("a.PNG", null))
        assertEquals("pdf", DropWriter.extensionOf("noext", "application/pdf"))
        assertEquals("bin", DropWriter.extensionOf("a.$%^", "text/x-weird"))
        assertNotNull(DropWriter.urlIn("see https://a.example/b now"))
    }
}
