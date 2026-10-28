package app.kanso.kanso

import android.app.Activity
import android.app.ActivityManager
import android.app.TaskInfo
import android.content.Context
import android.content.Intent
import android.os.Build
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.embedding.engine.FlutterEngineGroup
import io.flutter.embedding.engine.dart.DartExecutor
import io.flutter.FlutterInjector
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import java.lang.ref.WeakReference

/**
 * A note in a window of its own on Android: a task of its own, beside the
 * app or in freeform mode (`NoteWindowActivity`), with a Flutter engine of
 * its own in this process. The Dart side opens the one Rust library from all
 * of them, so typing and the lock session are shared.
 *
 * The Dart side talks to this over the `app.kanso/windows` channel, which
 * every engine has: the main engine's answers for the main window, and each
 * note window's for its own task. See `lib/window/` in the Flutter app.
 */
object NoteWindows {
    const val CHANNEL = "app.kanso/windows"
    const val ENTRY_POINT = "noteWindowMain"
    const val ENTRY_LIBRARY = "package:kanso/note_window_main.dart"

    const val EXTRA_NOTE_ID = "app.kanso.kanso.NOTE_ID"
    const val EXTRA_ENVIRONMENT = "app.kanso.kanso.ENVIRONMENT"
    const val EXTRA_LIBRARY_FOLDER = "app.kanso.kanso.LIBRARY_FOLDER"

    /** The note windows that are open, by note id. */
    private val windows = mutableMapOf<String, WeakReference<NoteWindowActivity>>()

    private var group: FlutterEngineGroup? = null
    private var mainChannel: MethodChannel? = null

    /** The engines that share resources: one per note window. */
    @Synchronized
    fun engineGroup(context: Context): FlutterEngineGroup {
        return group ?: FlutterEngineGroup(context.applicationContext).also { group = it }
    }

    /** The engine of a note's window, running the note window's entry point. */
    fun createEngine(context: Context, noteId: String, environment: String, folder: String): FlutterEngine {
        val entry =
            DartExecutor.DartEntrypoint(
                FlutterInjector.instance().flutterLoader().findAppBundlePath(),
                ENTRY_LIBRARY,
                ENTRY_POINT,
            )
        val options =
            FlutterEngineGroup.Options(context.applicationContext)
                .setDartEntrypoint(entry)
                .setDartEntrypointArgs(listOf(noteId, environment, folder))
        return engineGroup(context).createAndRunEngine(options)
    }

    /** Answers the channel for the main engine. */
    fun registerMain(engine: FlutterEngine, activity: Activity) {
        val channel = MethodChannel(engine.dartExecutor.binaryMessenger, CHANNEL)
        channel.setMethodCallHandler { call, result -> handle(call, result, activity, null) }
        mainChannel = channel
    }

    /** A note window came up, or went. */
    @Synchronized
    fun opened(window: NoteWindowActivity) {
        windows[window.noteId] = WeakReference(window)
    }

    @Synchronized
    fun closed(window: NoteWindowActivity) {
        if (windows[window.noteId]?.get() === window) {
            windows.remove(window.noteId)
        }
    }

    /** Answers the channel for a note window's engine. */
    fun registerNote(engine: FlutterEngine, activity: NoteWindowActivity) {
        val channel = MethodChannel(engine.dartExecutor.binaryMessenger, CHANNEL)
        channel.setMethodCallHandler { call, result -> handle(call, result, activity, activity) }
    }

    /** Whether this device can show more than one window: Android 10 and later always can. */
    fun supported(): Boolean = Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q

    private fun handle(call: MethodCall, result: MethodChannel.Result, context: Activity, note: NoteWindowActivity?) {
        when (call.method) {
            "supported" -> result.success(supported())
            "open" -> {
                val noteId = call.argument<String>("noteId")
                if (noteId.isNullOrEmpty()) {
                    result.error("invalid", "A window needs a note.", null)
                    return
                }
                open(
                    context,
                    noteId,
                    call.argument<String>("environment") ?: "production",
                    call.argument<String>("libraryFolder") ?: "",
                )
                result.success(null)
            }
            "closeThisWindow" -> {
                // The main window is never closed this way.
                note?.finishAndRemoveTask()
                result.success(null)
            }
            "closeWindowOf" -> {
                call.argument<String>("noteId")?.let { id -> windows[id]?.get()?.finishAndRemoveTask() }
                result.success(null)
            }
            "openWindows" -> {
                result.success(
                    windows.values.mapNotNull { it.get() }.map { window ->
                        mapOf("noteId" to window.noteId, "title" to window.title)
                    },
                )
            }
            "setTitle" -> {
                note?.let { window -> call.argument<String>("title")?.let(window::name) }
                result.success(null)
            }
            "hand" -> {
                hand(context, call.arguments)
                result.success(null)
            }
            else -> result.notImplemented()
        }
    }

    /** The task of note [noteId], when it has a window. */
    private fun taskOf(context: Context, noteId: String): ActivityManager.AppTask? {
        val manager = context.getSystemService(ActivityManager::class.java) ?: return null
        return manager.appTasks.firstOrNull { task ->
            val info: TaskInfo = task.taskInfo
            info.baseIntent.getStringExtra(EXTRA_NOTE_ID) == noteId
        }
    }

    /** Brings the note's window forward, or opens one beside the app. */
    fun open(context: Context, noteId: String, environment: String, folder: String) {
        val existing = taskOf(context, noteId)
        if (existing != null) {
            existing.moveToFront()
            return
        }
        val intent =
            Intent(context, NoteWindowActivity::class.java)
                .addFlags(
                    Intent.FLAG_ACTIVITY_LAUNCH_ADJACENT or
                        Intent.FLAG_ACTIVITY_NEW_DOCUMENT or
                        Intent.FLAG_ACTIVITY_MULTIPLE_TASK,
                )
                .putExtra(EXTRA_NOTE_ID, noteId)
                .putExtra(EXTRA_ENVIRONMENT, environment)
                .putExtra(EXTRA_LIBRARY_FOLDER, folder)
        context.startActivity(intent)
    }

    /** Hands a request of a note's window to the main window, and brings it forward. */
    private fun hand(context: Context, request: Any?) {
        mainChannel?.invokeMethod("request", request)
        val intent =
            Intent(context, MainActivity::class.java)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_REORDER_TO_FRONT)
        context.startActivity(intent)
    }
}
