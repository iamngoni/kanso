package app.kanso.kanso

import android.Manifest
import android.app.Activity
import android.content.Context
import android.content.pm.PackageManager
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

/**
 * `app.kanso.kanso/recording`: the microphone for the Record screen
 * (issue #38). The recorder itself is the Rust core's (cpal over AAudio),
 * which can't ask for RECORD_AUDIO; without it Android hands it silence or
 * an error. `prepare` asks once and answers whether it is granted.
 */
object KansoRecordingSession {
    private const val CHANNEL = "app.kanso.kanso/recording"
    private const val PERMISSION_REQUEST = 4802

    private var pending: MethodChannel.Result? = null
    // The application's context, not the activity: this object outlives
    // every activity (rotation makes a new one), and holding one would leak
    // it. Starting and stopping the service only needs the application.
    private var appContext: Context? = null

    fun register(activity: Activity, engine: FlutterEngine) {
        appContext = activity.applicationContext
        MethodChannel(engine.dartExecutor.binaryMessenger, CHANNEL)
            .setMethodCallHandler { call, result ->
                when (call.method) {
                    "prepare" -> prepare(activity, result)
                    "release" -> {
                        KansoRecordingService.stop(activity)
                        result.success(null)
                    }
                    else -> result.notImplemented()
                }
            }
    }

    private fun granted(activity: Activity): Boolean =
        activity.checkSelfPermission(Manifest.permission.RECORD_AUDIO) ==
            PackageManager.PERMISSION_GRANTED

    private fun prepare(activity: Activity, result: MethodChannel.Result) {
        if (granted(activity)) {
            KansoRecordingService.start(activity)
            result.success(true)
            return
        }
        // A second ask while the first is open answers the first "no".
        pending?.success(false)
        pending = result
        activity.requestPermissions(
            arrayOf(Manifest.permission.RECORD_AUDIO),
            PERMISSION_REQUEST,
        )
    }

    fun onPermissionResult(requestCode: Int, grantResults: IntArray) {
        if (requestCode != PERMISSION_REQUEST) return
        val answer = pending ?: return
        pending = null
        val ok = grantResults.isNotEmpty() &&
            grantResults[0] == PackageManager.PERMISSION_GRANTED
        if (ok) appContext?.let { KansoRecordingService.start(it) }
        answer.success(ok)
    }
}
