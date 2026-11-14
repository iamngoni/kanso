package app.kanso.kanso

import android.content.Context
import android.content.Intent
import android.net.Uri
import io.flutter.FlutterInjector
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.embedding.engine.FlutterEngineGroup
import io.flutter.embedding.engine.dart.DartExecutor

/**
 * The engines of the capture sheet over the current app (`CaptureActivity`):
 * each runs `captureMain` (`lib/capture_main.dart`), from the engine group
 * the note windows use, so one started while Kanso runs shares its code.
 *
 * One can be started ahead ([prewarm]), while the Quick Settings panel shows
 * the Add task tile, so the sheet is up as the tap lands. The activity takes
 * it ([take]) and destroys it when it goes, and the next one starts afresh.
 */
object CaptureEngines {
    const val ENTRY_POINT = "captureMain"
    const val ENTRY_LIBRARY = "package:kanso/capture_main.dart"

    /** The `AppEnvironment` name the engine runs in; production when absent. */
    const val EXTRA_ENVIRONMENT = "app.kanso.kanso.CAPTURE_ENVIRONMENT"

    /** The library folder, for tests; the app's own when absent. */
    const val EXTRA_LIBRARY_FOLDER = "app.kanso.kanso.CAPTURE_LIBRARY_FOLDER"

    /** The capture sheet in Note mode, as the Capture shortcut opens it. */
    const val CAPTURE_LINK = "kanso://open/new"

    /** The capture sheet in Task mode: the tile and the Add task shortcut. */
    const val ADD_TASK_LINK = "kanso://open/new?task=1"

    private const val PRODUCTION = "production"

    /** The engine started ahead, not yet shown. */
    private var warm: FlutterEngine? = null

    /**
     * Starts an engine for the next sheet, unless one is waiting already. Only
     * a head start: when it fails, the sheet starts its own.
     */
    @Synchronized
    fun prewarm(context: Context) {
        if (warm == null) {
            warm = runCatching { create(context, PRODUCTION, "") }.getOrNull()
        }
    }

    /**
     * The engine for a sheet: the one started ahead when it runs in the same
     * environment and library, or a new one.
     */
    @Synchronized
    fun take(context: Context, environment: String, folder: String): FlutterEngine {
        val ready = warm
        warm = null
        if (ready != null && environment == PRODUCTION && folder.isEmpty()) {
            return ready
        }
        ready?.destroy()
        return create(context, environment, folder)
    }

    private fun create(context: Context, environment: String, folder: String): FlutterEngine {
        // The tile can start one in a process no activity has run Flutter
        // in yet: the loader must know where the app's code is first.
        val loader = FlutterInjector.instance().flutterLoader()
        loader.startInitialization(context.applicationContext)
        loader.ensureInitializationComplete(context.applicationContext, null)
        val group = NoteWindows.engineGroup(context)
        val entry =
            DartExecutor.DartEntrypoint(
                loader.findAppBundlePath(),
                ENTRY_LIBRARY,
                ENTRY_POINT,
            )
        val options =
            FlutterEngineGroup.Options(context.applicationContext)
                .setDartEntrypoint(entry)
                .setDartEntrypointArgs(listOf(environment, folder))
        return group.createAndRunEngine(options)
    }

    /** Whether [link] asks for Task mode: `?task=1`, as the router reads it. */
    fun isTask(link: Uri?): Boolean = link?.getQueryParameter("task") == "1"

    /**
     * Opens the sheet on [link] over the app on screen, in a task of its
     * own that is never in the recents.
     */
    fun intent(context: Context, link: String): Intent =
        Intent(Intent.ACTION_VIEW, Uri.parse(link))
            .setClassName(context, CaptureActivity::class.java.name)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK)
}
