package app.kanso.kanso

import android.app.Application
import app.kanso.kanso_background.BackgroundSync
import app.kanso.kanso_background.SyncWorker
import app.kanso.kanso_lock.RustAnswer as LockAnswer
import app.kanso.kanso_passkeys.RustAnswer

/**
 * Sets the library up as the process starts: the Keystore's storage, the
 * native library, and the `Context` the certificate verifier needs. Then it
 * gives the plugins their way into the library and schedules the background
 * sync.
 *
 * Android runs this in every process of the app, WorkManager's included, so a
 * background sync finds the library ready.
 */
class KansoApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        KansoSecrets.attach(this)
        System.loadLibrary(KansoNative.LIBRARY)
        KansoNative.init(this)
        // The plugins are libraries and cannot see KansoNative: hand them the
        // calls.
        LockAnswer.install { ticket, status, secret -> KansoNative.biometricAnswer(ticket, status, secret) }
        BackgroundSync.install { path, budgetMs -> KansoNative.backgroundSync(path, budgetMs) }
        RustAnswer.install { ticket, json -> KansoNative.passkeyAnswer(ticket, json) }
        SyncWorker.schedulePeriodic(this)
        KansoCrashes.start(this)
    }
}
