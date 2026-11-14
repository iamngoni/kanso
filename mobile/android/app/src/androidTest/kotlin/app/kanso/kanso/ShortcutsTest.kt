package app.kanso.kanso

import android.content.Context
import android.content.Intent
import android.content.pm.ShortcutManager
import android.net.Uri
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.Until
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * The launcher's static shortcuts and the Add task tile's link (#390 slice
 * 19): Capture, Add task and Search come first. Capture and Add task open
 * the capture sheet over the app on screen (`CaptureActivity`, see
 * `CaptureOverAppTest`); the others open Kanso's links in `MainActivity`.
 * The `kanso://open/new?task=1` link itself still opens in the app, where
 * `integration_test/shortcuts_flow_test.dart` follows it.
 */
class ShortcutsTest {
    private val instrumentation = InstrumentationRegistry.getInstrumentation()
    private val app: Context = instrumentation.targetContext
    private val device = UiDevice.getInstance(instrumentation)

    private fun shortcuts() =
        app.getSystemService(ShortcutManager::class.java).manifestShortcuts

    @Test
    fun captureAddTaskAndSearchComeFirst() {
        val ids = shortcuts().sortedBy { it.rank }.map { it.id }
        assertEquals(listOf("new_note", "add_task", "search"), ids.take(3))
        assertTrue(ids.toString(), ids.containsAll(listOf("voice_note", "today_note")))
    }

    @Test
    fun eachOpensItsLink() {
        val capture = CaptureActivity::class.java.name
        val main = MainActivity::class.java.name
        val links = mapOf(
            "new_note" to (CaptureEngines.CAPTURE_LINK to capture),
            "add_task" to (TaskTileService.ADD_TASK_LINK to capture),
            "search" to ("kanso://open/search" to main),
            "voice_note" to ("kanso://open/new?voice=1" to main),
            "today_note" to ("kanso://open/daily" to main),
        )
        val byId = shortcuts().associateBy { it.id }
        for ((id, target) in links) {
            val (link, activity) = target
            val shortcut = byId[id]
            assertNotNull(id, shortcut)
            val intent = shortcut!!.intent!!
            assertEquals(id, Intent.ACTION_VIEW, intent.action)
            assertEquals(id, link, intent.dataString)
            assertEquals(id, activity, intent.component?.className)
        }
    }

    @Test
    fun theTileAndTheShortcutOpenTheSameLink() {
        val tile = TaskTileService.addTaskIntent(app)
        val shortcut = shortcuts().first { it.id == "add_task" }.intent!!
        assertEquals(shortcut.dataString, tile.dataString)
        assertEquals(shortcut.component?.className, tile.component?.className)
        val resolved = app.packageManager.resolveActivity(tile, 0)
        assertEquals(CaptureActivity::class.java.name, resolved?.activityInfo?.name)
    }

    @Test
    fun theAddTaskLinkStillOpensInTheApp() {
        // Another app's link, or `adb shell am start -d`: Kanso's own screen,
        // where the router shows the Task sheet. The capture activity has no
        // link of its own.
        val link = Intent(Intent.ACTION_VIEW, Uri.parse(TaskTileService.ADD_TASK_LINK))
            .setPackage(app.packageName)
        val resolved = app.packageManager.resolveActivity(link, 0)
        assertEquals(MainActivity::class.java.name, resolved?.activityInfo?.name)
        device.pressHome()
        app.startActivity(link.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
        assertTrue(
            "Kanso did not come forward for the Add task link.",
            device.wait(Until.hasObject(By.pkg(app.packageName).depth(0)), 15_000),
        )
        device.pressHome()
    }
}
