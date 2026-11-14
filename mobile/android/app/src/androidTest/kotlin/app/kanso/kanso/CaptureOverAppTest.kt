package app.kanso.kanso

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.ActivityInfo
import android.database.sqlite.SQLiteDatabase
import android.os.ParcelFileDescriptor
import android.provider.Settings
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.By
import androidx.test.uiautomator.BySelector
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.UiObject2
import androidx.test.uiautomator.Until
import java.io.File
import org.junit.After
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

/**
 * The capture sheet over the current app (#390 slice 19): the Add task tile
 * and the Capture and Add task shortcuts open `CaptureActivity`, a
 * see-through window in a task of its own, over whatever app is on screen.
 * That app stays visible beneath it and Kanso's own screen does not come
 * forward; a task written, or the sheet dismissed, finishes it, and the
 * other app is in front again.
 *
 * Settings is the other app. The task is read back from the library, as
 * `CaptureShareTest` reads the shares.
 */
class CaptureOverAppTest {
    private val instrumentation = InstrumentationRegistry.getInstrumentation()
    private val device = UiDevice.getInstance(instrumentation)
    private val app: Context = instrumentation.targetContext
    private val kanso = app.packageName
    private val library = File(app.filesDir, "kanso")
    private val settings = "com.android.settings"

    @Before
    fun setUp() {
        // The library is made by the app's first launch.
        if (!File(library, "library.db").exists()) {
            val launch = app.packageManager.getLaunchIntentForPackage(kanso)!!
            launch.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            app.startActivity(launch)
            waitFor("the library to be made") { File(library, "library.db").exists() }
        }
        device.pressHome()
        app.startActivity(Intent(Settings.ACTION_SETTINGS).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
        assertTrue("Settings did not open.", device.wait(Until.hasObject(By.pkg(settings).depth(0)), 20_000))
    }

    @After
    fun tearDown() {
        device.pressHome()
    }

    @Test
    fun theWindowIsSeeThroughAndOutOfTheRecents() {
        val info = app.packageManager.getActivityInfo(ComponentName(app, CaptureActivity::class.java), 0)
        assertTrue("not excluded from the recents", info.flags and ActivityInfo.FLAG_EXCLUDE_FROM_RECENTS != 0)
        assertNotEquals("the capture task would be Kanso's", kanso, info.taskAffinity)
        val theme = app.createPackageContext(kanso, 0).apply { setTheme(info.themeResource) }.theme
        val attrs = theme.obtainStyledAttributes(intArrayOf(android.R.attr.windowIsTranslucent))
        try {
            assertTrue("the window is not translucent", attrs.getBoolean(0, false))
        } finally {
            attrs.recycle()
        }
    }

    @Test
    fun aTaskWrittenOverAnotherAppFinishesBackToIt() {
        app.startActivity(TaskTileService.addTaskIntent(app))
        val field = waitForObject(By.pkg(kanso).clazz("android.widget.EditText"), "the Task field")
        assertOverSettings()

        val words = "Kanso over-app test ${System.currentTimeMillis()}"
        field.click()
        field.text = words
        waitForLabel("Add task").click()

        assertTrue("the sheet did not finish", device.wait(Until.gone(By.pkg(kanso)), 20_000))
        assertTrue("Settings is not in front again", device.hasObject(By.pkg(settings).depth(0)))
        waitFor("the task to be written") {
            query("select count(*) from docs where type = 'note' and json like '%$words%'") { it.getInt(0) }
                .single() == 1
        }
    }

    @Test
    fun anEngineStartedAheadIsUsed() {
        // As the Quick Settings panel does while it shows the tile.
        instrumentation.runOnMainSync { CaptureEngines.prewarm(app) }
        app.startActivity(TaskTileService.addTaskIntent(app))
        waitForObject(By.pkg(kanso).clazz("android.widget.EditText"), "the Task field")
        assertOverSettings()
        // Back closes the keyboard when it is up, then the sheet; never more,
        // or Settings would be left too.
        repeat(2) {
            if (device.hasObject(By.pkg(kanso))) {
                device.pressBack()
                device.wait(Until.gone(By.pkg(kanso)), 3_000)
            }
        }
        assertTrue("the sheet did not finish", device.wait(Until.gone(By.pkg(kanso)), 20_000))
        assertTrue("Settings is not in front again", device.hasObject(By.pkg(settings).depth(0)))
    }

    @Test
    fun captureOpensNoteModeAndDismissedFinishes() {
        app.startActivity(CaptureEngines.intent(app, CaptureEngines.CAPTURE_LINK))
        waitForLabel("Take photo")
        assertOverSettings()
        device.pressBack()
        assertTrue("the sheet did not finish", device.wait(Until.gone(By.pkg(kanso)), 20_000))
        assertTrue(device.hasObject(By.pkg(settings).depth(0)))
    }

    // MARK: - Steps

    /** The capture task is in front, Settings is still drawn beneath it, and Kanso's own screen is not. */
    private fun assertOverSettings() {
        val records = activityRecords()
        val capture = records.entries.firstOrNull { it.key.endsWith("/.CaptureActivity") }
        assertNotNull("no capture activity in $records", capture)
        assertTrue("the capture sheet is not shown", capture!!.value)
        assertTrue(
            "Settings is not visible beneath the sheet: $records",
            records.any { (name, visible) -> name.startsWith("$settings/") && visible },
        )
        assertFalse(
            "Kanso's own screen came forward: $records",
            records.any { (name, visible) -> name.endsWith("/.MainActivity") && name.startsWith(kanso) && visible },
        )
    }

    /** Each activity's component, with whether the system has it visible, from `dumpsys`. */
    private fun activityRecords(): Map<String, Boolean> {
        val out = shell("dumpsys activity activities")
        val records = linkedMapOf<String, Boolean>()
        var current: String? = null
        val record = Regex("""Hist\s+#\d+: ActivityRecord\{\S+ \S+ (\S+) """)
        for (line in out.lineSequence()) {
            record.find(line)?.let { current = it.groupValues[1] }
            val name = current
            if (name != null && line.contains("mVisibleRequested=")) {
                records.putIfAbsent(name, line.contains("mVisibleRequested=true"))
                current = null
            }
        }
        return records
    }

    private fun shell(command: String): String {
        val fd = instrumentation.uiAutomation.executeShellCommand(command)
        return ParcelFileDescriptor.AutoCloseInputStream(fd).bufferedReader().use { it.readText() }
    }

    /** A Flutter widget of Kanso's by its label, which Flutter gives as its description or its text. */
    private fun waitForLabel(label: String): UiObject2 {
        var found: UiObject2? = null
        waitFor("'$label' to show", 20_000) {
            found = device.findObject(By.pkg(kanso).desc(label)) ?: device.findObject(By.pkg(kanso).text(label))
            found != null
        }
        return found!!
    }

    private fun waitForObject(selector: BySelector, what: String): UiObject2 {
        val found = device.wait(Until.findObject(selector), 20_000)
        assertNotNull("$what did not show", found)
        return found!!
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
