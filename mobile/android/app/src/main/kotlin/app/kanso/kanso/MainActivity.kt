package app.kanso.kanso

import android.content.Intent
import android.os.Bundle
import androidx.glance.appwidget.updateAll
import io.flutter.embedding.android.FlutterFragmentActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch

/** A `FragmentActivity`, which `BiometricPrompt` needs. */
class MainActivity : FlutterFragmentActivity() {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    private val files by lazy { KansoFiles(this) }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        if (savedInstanceState == null) {
            files.handleIntent(intent)
            KansoReminders.handleOpen(this, intent)
        }
    }

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        KansoCrashes.register(flutterEngine)
        files.register(flutterEngine)
        KansoReminders.register(this, flutterEngine)
        // `app.kanso.kanso/widgets`: the app asks for the home-screen widget
        // to be drawn again once it has written its colors (`widget.json`).
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, "app.kanso.kanso/widgets")
            .setMethodCallHandler { call, result ->
                if (call.method != "reload") {
                    result.notImplemented()
                    return@setMethodCallHandler
                }
                scope.launch {
                    try {
                        KansoWidget().updateAll(applicationContext)
                        result.success(null)
                    } catch (error: Exception) {
                        result.error("widgets", error.message, null)
                    }
                }
            }
        // `app.kanso.kanso/note_windows`: a note in a window of its own
        // (NoteWindows.kt), launched beside this one.
        NoteWindows.registerMain(flutterEngine, this)
    }

    override fun cleanUpFlutterEngine(flutterEngine: FlutterEngine) {
        KansoReminders.unregister()
        super.cleanUpFlutterEngine(flutterEngine)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        files.handleIntent(intent)
        KansoReminders.handleOpen(this, intent)
    }

    override fun onResume() {
        super.onResume()
        KansoReminders.inFront = true
    }

    override fun onPause() {
        KansoReminders.inFront = false
        super.onPause()
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray,
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        KansoReminders.onPermissionResult(requestCode, grantResults)
    }

    @Deprecated("The document picker result comes back through here.")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        if (!files.handleResult(requestCode, resultCode, data)) {
            @Suppress("DEPRECATION")
            super.onActivityResult(requestCode, resultCode, data)
        }
    }
}
