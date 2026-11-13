package app.kanso.kanso

import android.content.Context
import android.content.Intent
import android.content.pm.ShortcutManager
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
 * 19): Capture, Add task and Search come first and open Kanso's links in
 * `MainActivity`, and the Add task link brings Kanso forward. What the app
 * does with the link (the Task sheet over the current screen) is
 * `integration_test/shortcuts_flow_test.dart`.
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
    fun eachOpensItsLinkInMainActivity() {
        val links = mapOf(
            "new_note" to "kanso://open/new",
            "add_task" to TaskTileService.ADD_TASK_LINK,
            "search" to "kanso://open/search",
            "voice_note" to "kanso://open/new?voice=1",
            "today_note" to "kanso://open/daily",
        )
        val byId = shortcuts().associateBy { it.id }
        for ((id, link) in links) {
            val shortcut = byId[id]
            assertNotNull(id, shortcut)
            val intent = shortcut!!.intent!!
            assertEquals(id, Intent.ACTION_VIEW, intent.action)
            assertEquals(id, link, intent.dataString)
            assertEquals(id, MainActivity::class.java.name, intent.component?.className)
        }
    }

    @Test
    fun theTileAndTheShortcutOpenTheSameLink() {
        val tile = TaskTileService.addTaskIntent(app)
        val shortcut = shortcuts().first { it.id == "add_task" }.intent!!
        assertEquals(shortcut.dataString, tile.dataString)
        assertEquals(shortcut.component?.className, tile.component?.className)
        val resolved = app.packageManager.resolveActivity(tile, 0)
        assertEquals(MainActivity::class.java.name, resolved?.activityInfo?.name)
    }

    @Test
    fun theAddTaskLinkBringsKansoForward() {
        device.pressHome()
        app.startActivity(TaskTileService.addTaskIntent(app))
        assertTrue(
            "Kanso did not come forward for the Add task link.",
            device.wait(Until.hasObject(By.pkg(app.packageName).depth(0)), 15_000),
        )
    }
}
