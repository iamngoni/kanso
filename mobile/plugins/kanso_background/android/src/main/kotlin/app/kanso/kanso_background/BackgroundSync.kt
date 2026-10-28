package app.kanso.kanso_background

import android.content.Context
import java.io.File

/**
 * The app's side of the sync with no app: the call into the Rust library
 * (`KansoNative.backgroundSync` in the app module, which this plugin cannot
 * see).
 *
 * `KansoApplication.onCreate` installs it, after `KansoNative.init`. A
 * WorkManager job runs in the app's process after `Application.onCreate`, so
 * it is always there; if not, the sync answers [INTERNAL].
 */
fun interface BackgroundSyncEntry {
    /** Syncs the library file at [path] within [budgetMs]; the codes of `background.rs`. */
    fun sync(path: String, budgetMs: Int): Int
}

object BackgroundSync {
    /** The budget one job gets, in milliseconds. */
    const val BUDGET_MS = 120_000

    /** The result for anything outside the sync itself, as the Rust side numbers it. */
    const val INTERNAL = -5

    @Volatile
    private var entry: BackgroundSyncEntry? = null

    /** Installs the call into the library. */
    @JvmStatic
    fun install(entry: BackgroundSyncEntry) {
        this.entry = entry
    }

    /**
     * The library's database file: the one the app opens. path_provider's
     * application support folder on Android is `Context.getFilesDir()`.
     */
    @JvmStatic
    fun libraryPath(context: Context): String =
        File(File(context.filesDir, "kanso"), "library.db").path

    /** Syncs once, on the calling thread. */
    @JvmStatic
    fun run(context: Context, budgetMs: Int = BUDGET_MS): Int {
        val entry = entry ?: return INTERNAL
        return try {
            entry.sync(libraryPath(context), budgetMs)
        } catch (error: Throwable) {
            INTERNAL
        }
    }
}
