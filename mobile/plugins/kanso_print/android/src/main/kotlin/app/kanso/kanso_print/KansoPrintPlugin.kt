package app.kanso.kanso_print

import android.content.Context
import android.os.Handler
import android.os.Looper
import io.flutter.embedding.engine.plugins.FlutterPlugin
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException
import kotlin.coroutines.suspendCoroutine

/** The Dart side of `KansoPrint.htmlToPdf`. */
class KansoPrintPlugin :
    FlutterPlugin,
    PrintHostApi {
    private var context: Context? = null

    override fun onAttachedToEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        context = binding.applicationContext
        PrintHostApi.setUp(binding.binaryMessenger, this)
    }

    override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        PrintHostApi.setUp(binding.binaryMessenger, null)
        context = null
    }

    override suspend fun htmlToPdf(
        html: String,
        outputPath: String,
        paper: PrintPaper,
        marginPt: Double,
    ): Long {
        val context = context ?: throw FlutterError("detached", "The plugin is not attached.", null)
        return suspendCoroutine { continuation ->
            // WebView lives on the main thread; the job reports back once.
            Handler(Looper.getMainLooper()).post {
                try {
                    PdfJob(context, html, outputPath, paper, marginPt) { result ->
                        result.fold(
                            { continuation.resume(it.toLong()) },
                            { continuation.resumeWithException(it) },
                        )
                    }.start()
                } catch (e: Throwable) {
                    continuation.resumeWithException(FlutterError("failed", e.message ?: e.toString(), null))
                }
            }
        }
    }
}
