package app.kanso.kanso

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.os.Bundle

/**
 * Starts or stops a voice note for the Quick Settings tile and the widget.
 *
 * Android 14 and later refuse the microphone to a foreground service started
 * from the background, which a tile's or a widget's own start counts as. This
 * activity is invisible and lasts a moment: started while it is on screen, the
 * service is a foreground start and keeps the microphone after it closes.
 */
class VoiceCaptureActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        if (VoiceState.recording) {
            try {
                startService(VoiceCaptureService.toggleIntent(this).setAction(VoiceCaptureService.ACTION_STOP))
            } catch (_: RuntimeException) {
                VoiceState.set(false)
            }
        } else if (!VoiceCaptureService.start(this)) {
            // The permission is missing or the start was refused: the app's
            // voice note asks for it.
            startActivity(VoiceCaptureService.voiceLink(this))
        }
        finish()
        overridePendingTransition(0, 0)
    }

    companion object {
        fun intent(context: Context): Intent =
            Intent(context, VoiceCaptureActivity::class.java)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_NO_ANIMATION)
    }
}
