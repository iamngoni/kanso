package app.kanso.kanso

import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService

/**
 * The Quick Settings tile "Add task": a tap opens Kanso with
 * `kanso://open/new?task=1`, which shows the capture sheet in Task mode over
 * the screen the app was on (the Dart router, `deep_links.dart`). The same
 * link is the launcher's Add task shortcut (`res/xml/shortcuts.xml`).
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
        const val ADD_TASK_LINK = "kanso://open/new?task=1"

        // VoiceTileService's pending intent is 4.
        private const val REQUEST_CODE = 5

        /** Opens `MainActivity` on [ADD_TASK_LINK], as the shortcut does. */
        fun addTaskIntent(context: Context): Intent =
            Intent(Intent.ACTION_VIEW, Uri.parse(ADD_TASK_LINK))
                .setClassName(context, MainActivity::class.java.name)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
    }
}
