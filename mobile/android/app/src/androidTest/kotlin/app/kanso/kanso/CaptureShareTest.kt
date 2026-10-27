package app.kanso.kanso

import android.content.ClipData
import android.content.Context
import android.content.Intent
import android.database.sqlite.SQLiteDatabase
import android.net.Uri
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.UiDevice
import java.io.File
import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

/**
 * Shares a link, an image and a PDF to [ShareActivity] with Kanso not running
 * and reads the library to see the notes (#48 step 10).
 *
 * The instrumentation lives in Kanso's process, so the tests can't stop it
 * (`am force-stop` would end the run); `scripts/mobile-capture-android.sh`
 * shares with the process stopped too, through `am start`.
 *
 * The files are served by [CaptureProvider] (declared in the test manifest)
 * and handed over with a read grant, as the app that shares them would. The test runs in Kanso's process, so
 * `filesDir` is the library's folder. Run by
 * `scripts/mobile-capture-android.sh`.
 */
class CaptureShareTest {
    private val instrumentation = InstrumentationRegistry.getInstrumentation()
    private val device = UiDevice.getInstance(instrumentation)
    private val app: Context = instrumentation.targetContext
    private val tests: Context = instrumentation.context
    private val library = File(app.filesDir, "kanso")
    private val inbox = File(library, "inbox")
    private val authority = "app.kanso.kanso.test.captureprovider"

    /** The notes in the Inbox before the test, so a run only counts its own. */
    private var before = emptySet<String>()

    @Before
    fun setUp() {
        // The library is made by the app's first launch: launch it once if
        // this is a fresh install. The tests then share without Kanso's
        // screen open; the share activity starts no engine of its own.
        if (!File(library, "library.db").exists()) {
            val launch = app.packageManager.getLaunchIntentForPackage(app.packageName)!!
            launch.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            app.startActivity(launch)
            waitFor("the library to be made") { File(library, "library.db").exists() }
            device.pressHome()
        }
        before = captureNoteIds()
    }

    @Test
    fun linkInText() {
        share(Intent(Intent.ACTION_SEND).setType("text/plain")
            .putExtra(Intent.EXTRA_SUBJECT, "Kanso capture link test")
            .putExtra(Intent.EXTRA_TEXT, "https://example.com/capture-link-test"))
        val note = newNote()
        assertEquals("Kanso capture link test", note.getString("title"))
        assertEquals("book:inbox", note.getString("bookId"))
        assertTrue(note.getString("body"), note.getString("body").contains("(https://example.com/capture-link-test)"))
        assertInboxEmpty()
    }

    @Test
    fun image() {
        val photo = asset("Kanso-test-photo.png")
        share(Intent(Intent.ACTION_SEND).setType("image/png").putExtra(Intent.EXTRA_STREAM, photo), photo)
        val note = newNote()
        assertEquals("book:inbox", note.getString("bookId"))
        assertTrue(note.getString("body"), note.getString("body").contains("![](inkdrop://file:"))
        assertEquals(listOf("image/png"), fileTypes())
        assertInboxEmpty()
    }

    @Test
    fun pdf() {
        val report = asset("Quarterly-report.pdf")
        share(Intent(Intent.ACTION_SEND).setType("application/pdf").putExtra(Intent.EXTRA_STREAM, report), report)
        val note = newNote()
        assertEquals("Quarterly-report", note.getString("title"))
        assertEquals("book:inbox", note.getString("bookId"))
        assertTrue(note.getString("body"), note.getString("body").contains("[Quarterly-report.pdf](inkdrop://file:"))
        assertEquals(listOf("application/pdf"), fileTypes())
        assertInboxEmpty()
    }

    @Test
    fun imageAndPdf() {
        val one = asset("Kanso-test-photo.png")
        val two = asset("Quarterly-report.pdf")
        share(
            Intent(Intent.ACTION_SEND_MULTIPLE).setType("*/*")
                .putParcelableArrayListExtra(Intent.EXTRA_STREAM, arrayListOf(one, two)),
            one, two,
        )
        val note = newNote()
        assertTrue(note.getString("body"), note.getString("body").contains("![](inkdrop://file:"))
        assertTrue(note.getString("body"), note.getString("body").contains("[Quarterly-report.pdf](inkdrop://file:"))
        assertEquals(listOf("application/pdf", "image/png"), fileTypes().sorted())
        assertInboxEmpty()
    }

    // MARK: - Steps

    /** The provider's content URI for a test asset. */
    private fun asset(name: String): Uri = Uri.parse("content://$authority/$name")

    /** Starts the share the way another app does and waits for Kanso's activity to finish. */
    private fun share(intent: Intent, vararg files: Uri) {
        intent.setClassName(app.packageName, "app.kanso.kanso.ShareActivity")
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_GRANT_READ_URI_PERMISSION)
        if (files.isNotEmpty()) {
            val clip = ClipData.newRawUri("shared", files.first())
            files.drop(1).forEach { clip.addItem(ClipData.Item(it)) }
            intent.clipData = clip
        }
        tests.startActivity(intent)
        waitFor("the share to be saved") { (captureNoteIds() - before).isNotEmpty() }
    }

    private fun captureNoteIds(): Set<String> =
        query("select id from docs where id like 'note:cap-%' and deleted = 0") { it.getString(0) }.toSet()

    /** The one note this test's share made, as its JSON. */
    private fun newNote(): JSONObject {
        val ids = captureNoteIds() - before
        assertEquals("one new capture note: $ids", 1, ids.size)
        return JSONObject(query("select json from docs where id = '${ids.first()}'") { it.getString(0) }.single())
    }

    /** The content types of the files the new note links, in order. */
    private fun fileTypes(): List<String> {
        val id = (captureNoteIds() - before).first().removePrefix("note:cap-")
        return query("select json from docs where type = 'file' and id like 'file:cap-$id-%' order by id") {
            JSONObject(it.getString(0)).getString("contentType")
        }
    }

    private fun assertInboxEmpty() {
        val left = inbox.list()?.toList() ?: emptyList()
        assertTrue("the inbox still holds $left", left.isEmpty())
    }

    private fun <T> query(sql: String, read: (android.database.Cursor) -> T): List<T> {
        val db = SQLiteDatabase.openDatabase(File(library, "library.db").path, null, SQLiteDatabase.OPEN_READONLY)
        db.use {
            it.rawQuery(sql, null).use { cursor ->
                val rows = mutableListOf<T>()
                while (cursor.moveToNext()) rows += read(cursor)
                return rows
            }
        }
    }

    private fun waitFor(what: String, timeoutMs: Long = 30_000, done: () -> Boolean) {
        val end = System.currentTimeMillis() + timeoutMs
        while (System.currentTimeMillis() < end) {
            if (try { done() } catch (_: Exception) { false }) return
            Thread.sleep(250)
        }
        throw AssertionError("Timed out waiting for $what")
    }
}
