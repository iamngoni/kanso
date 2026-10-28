package app.kanso.kanso_background

import android.content.Context
import io.flutter.embedding.engine.plugins.FlutterPlugin
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/** The Dart side of the background sync: `KansoBackground.runNow` and `schedule`. */
class KansoBackgroundPlugin :
    FlutterPlugin,
    BackgroundHostApi {
    private var context: Context? = null

    override fun onAttachedToEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        context = binding.applicationContext
        BackgroundHostApi.setUp(binding.binaryMessenger, this)
    }

    override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        BackgroundHostApi.setUp(binding.binaryMessenger, null)
        context = null
    }

    override suspend fun runNow(): Long {
        val context = context ?: return BackgroundSync.INTERNAL.toLong()
        return withContext(Dispatchers.IO) { BackgroundSync.run(context).toLong() }
    }

    override fun schedule() {
        val context = context ?: return
        SyncWorker.schedulePeriodic(context)
    }
}
