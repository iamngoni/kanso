package app.kanso.kanso

import android.app.PendingIntent
import android.os.Build
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService

/**
 * The Quick Settings tile "Voice note": active while recording, and a tap
 * starts or stops [VoiceCaptureService]. It works from the lock screen, as the
 * recording needs no unlock.
 */
class VoiceTileService : TileService() {
    private val follow: (Boolean) -> Unit = { render() }

    override fun onStartListening() {
        VoiceState.listen(follow)
        render()
    }

    override fun onStopListening() {
        VoiceState.unlisten(follow)
    }

    override fun onClick() {
        if (VoiceState.recording) {
            try {
                startService(VoiceCaptureService.toggleIntent(this).setAction(VoiceCaptureService.ACTION_STOP))
            } catch (_: RuntimeException) {
                VoiceState.set(false)
            }
        } else if (!VoiceCaptureService.start(this)) {
            openApp()
        }
        render()
    }

    private fun render() {
        val tile = qsTile ?: return
        val recording = VoiceState.recording
        tile.state = when (VoiceCapture.tileMode(recording)) {
            VoiceTileMode.ACTIVE -> Tile.STATE_ACTIVE
            VoiceTileMode.INACTIVE -> Tile.STATE_INACTIVE
        }
        tile.subtitle = getString(if (recording) R.string.tile_voice_recording else R.string.tile_voice_idle)
        tile.updateTile()
    }

    /** The permission is missing or the start was refused: open the app's voice note. */
    @Suppress("DEPRECATION")
    private fun openApp() {
        val link = VoiceCaptureService.voiceLink(this)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            startActivityAndCollapse(
                PendingIntent.getActivity(this, 3, link, PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT),
            )
        } else {
            startActivityAndCollapse(link)
        }
    }
}
