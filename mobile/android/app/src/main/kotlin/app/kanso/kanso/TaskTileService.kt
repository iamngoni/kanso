package app.kanso.kanso

import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.os.Build
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService

/**
 * The Quick Settings tile "Add task": a tap opens the capture sheet in Task
 * mode over the app on screen (`CaptureActivity` on
 * `kanso://open/new?task=1`), without bringing Kanso forward. The launcher's
 * Add task shortcut (`res/xml/shortcuts.xml`) opens the same.
 *
 * While the panel shows the tile, the sheet's engine is started ahead, so
 * it is up as soon as the tile is tapped.
 *
 * The task goes into the notes, so on the lock screen the phone is unlocked
 * first.
 */
class TaskTileService : TileService() {
    override fun onStartListening() {
        val tile = qsTile ?: return
        // An action, not a switch: it is never "on".
        tile.state = Tile.STATE_INACTIVE
        tile.updateTile()
        CaptureEngines.prewarm(this)
    }

    override fun onClick() {
        if (isLocked) {
            unlockAndRun { open() }
        } else {
            open()
        }
    }

    private fun open() {
        val intent = addTaskIntent(this)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            val pending = PendingIntent.getActivity(
                this, REQUEST_CODE, intent,
                PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
            )
            startActivityAndCollapse(pending)
        } else {
            @Suppress("DEPRECATION")
            startActivityAndCollapse(intent)
        }
    }

    companion object {
        /** The link the tile and the Add task shortcut open. */
        const val ADD_TASK_LINK = CaptureEngines.ADD_TASK_LINK

        // VoiceTileService's pending intent is 4.
        private const val REQUEST_CODE = 5

        /** Opens `CaptureActivity` on [ADD_TASK_LINK], as the shortcut does. */
        fun addTaskIntent(context: Context): Intent =
            CaptureEngines.intent(context, ADD_TASK_LINK)
    }
}
