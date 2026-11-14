package app.kanso.kanso

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.android.FlutterActivityLaunchConfigs.BackgroundMode
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

/**
 * The capture sheet over the current app (#390 slice 19): the Add task tile,
 * and the Capture and Add task launcher shortcuts. A see-through window in a
 * task of its own (another affinity, never in the recents), so the app the
 * person was in stays on screen beneath it and Kanso's own task is not
 * brought forward. Its engine runs `captureMain`, which shows only the
 * capture sheet in the mode the link asks for (`?task=1` is Task mode) and
 * saves through the same library calls as the sheet inside the app.
 *
 * Saved or dismissed, it finishes, and the app beneath is where the person
 * is. It also finishes when it leaves the screen, except while a picker it
 * opened (camera, photos, PDF) is in front.
 *
 * The Dart side talks to it over `app.kanso/capture`: `mode` (`task` or
 * `note`), `finish`, and `open` (a link to open in Kanso itself, such as a
 * new note, after which the sheet goes).
 */
class CaptureActivity : FlutterActivity() {
    /** Whether a picker this activity started is in front of it. */
    private var awaitingResult = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // No slide in or out of a task: the sheet animates itself.
        @Suppress("DEPRECATION")
        overridePendingTransition(0, 0)
    }

    override fun provideFlutterEngine(context: Context): FlutterEngine =
        CaptureEngines.take(
            context,
            intent.getStringExtra(CaptureEngines.EXTRA_ENVIRONMENT) ?: "production",
            intent.getStringExtra(CaptureEngines.EXTRA_LIBRARY_FOLDER) ?: "",
        )

    // The engine is this sheet's: it goes when the sheet does.
    override fun shouldDestroyEngineWithHost(): Boolean = true

    // A transparent surface, so the app beneath shows around the sheet.
    override fun getBackgroundMode(): BackgroundMode = BackgroundMode.transparent

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        KansoCrashes.register(flutterEngine)
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CHANNEL)
            .setMethodCallHandler { call, result ->
                when (call.method) {
                    "mode" -> result.success(if (CaptureEngines.isTask(intent.data)) "task" else "note")
                    "finish" -> {
                        result.success(null)
                        close()
                    }
                    "open" -> {
                        val link = call.arguments as? String
                        result.success(null)
                        if (link != null) {
                            openInKanso(link)
                        }
                        close()
                    }
                    else -> result.notImplemented()
                }
            }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
    }

    @Deprecated("Pickers' results come back through here.")
    override fun startActivityForResult(intent: Intent, requestCode: Int, options: Bundle?) {
        awaitingResult = true
        @Suppress("DEPRECATION")
        super.startActivityForResult(intent, requestCode, options)
    }

    @Deprecated("Pickers' results come back through here.")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        awaitingResult = false
        @Suppress("DEPRECATION")
        super.onActivityResult(requestCode, resultCode, data)
    }

    override fun onStop() {
        super.onStop()
        // Home, another app, the screen off: the quick sheet is not kept.
        if (!awaitingResult && !isChangingConfigurations && !isFinishing) {
            close()
        }
    }

    private fun openInKanso(link: String) {
        startActivity(
            Intent(Intent.ACTION_VIEW, Uri.parse(link))
                .setClassName(this, MainActivity::class.java.name)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
        )
    }

    private fun close() {
        if (!isFinishing) {
            finishAndRemoveTask()
            @Suppress("DEPRECATION")
            overridePendingTransition(0, 0)
        }
    }

    companion object {
        const val CHANNEL = "app.kanso/capture"
    }
}
