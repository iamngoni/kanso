package app.kanso.kanso

import android.app.ActivityManager
import android.content.Context
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine

/**
 * A note in a window of its own: a task per note (document launch mode),
 * with an engine of its own from the app's engine group running
 * `noteWindowMain` on the note. The note and where the library is travel in
 * the intent, so the task comes back after the process was stopped.
 *
 * Its label in the recents and the window list is the note's title, or
 * "Untitled", or "Locked note" for a locked note (the Dart side names it
 * with `setTitle`): a locked note's title is never put there.
 */
class NoteWindowActivity : FlutterActivity() {
    private var engine: FlutterEngine? = null

    /** The note this window shows. */
    val noteId: String
        get() = intent.getStringExtra(NoteWindows.EXTRA_NOTE_ID) ?: ""

    /** The name the task has been given. */
    var title: String = ""
        private set

    override fun onCreate(savedInstanceState: android.os.Bundle?) {
        super.onCreate(savedInstanceState)
        NoteWindows.opened(this)
    }

    override fun onDestroy() {
        NoteWindows.closed(this)
        super.onDestroy()
    }

    override fun provideFlutterEngine(context: Context): FlutterEngine? {
        val noteId = intent.getStringExtra(NoteWindows.EXTRA_NOTE_ID) ?: return null
        val created =
            NoteWindows.createEngine(
                context,
                noteId,
                intent.getStringExtra(NoteWindows.EXTRA_ENVIRONMENT) ?: "production",
                intent.getStringExtra(NoteWindows.EXTRA_LIBRARY_FOLDER) ?: "",
            )
        engine = created
        return created
    }

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        KansoCrashes.register(flutterEngine)
        NoteWindows.registerNote(flutterEngine, this)
    }

    // The engine is this window's: it goes when the window does.
    override fun shouldDestroyEngineWithHost(): Boolean = true

    /** Names the task, for the recents and the window list. */
    fun name(title: String) {
        this.title = title
        setTaskDescription(ActivityManager.TaskDescription(title))
    }
}
