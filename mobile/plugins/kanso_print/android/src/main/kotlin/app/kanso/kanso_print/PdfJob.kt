package app.kanso.kanso_print

import android.content.Context
import android.graphics.pdf.PdfRenderer
import android.os.Handler
import android.os.Looper
import android.os.ParcelFileDescriptor
import android.print.KansoPdfWriter
import android.print.PrintAttributes
import android.webkit.WebView
import android.webkit.WebViewClient
import java.io.File

/**
 * One page printed to one PDF: an offscreen `WebView` (no JavaScript) loads
 * the HTML, and on `onPageFinished` the print adapter writes the pages to
 * the file through [KansoPdfWriter]. Main thread only; [done] is called once.
 */
internal class PdfJob(
    context: Context,
    private val html: String,
    private val outputPath: String,
    private val paper: PrintPaper,
    private val marginPt: Double,
    private val done: (Result<Int>) -> Unit,
) {
    private val handler = Handler(Looper.getMainLooper())
    private val webView = WebView(context)
    private var finished = false

    private val timeout = Runnable {
        finish(Result.failure(FlutterError("timeout", "The page took too long to lay out.", null)))
    }

    fun start() {
        handler.postDelayed(timeout, TIMEOUT_MS)
        webView.settings.javaScriptEnabled = false
        webView.webViewClient =
            object : WebViewClient() {
                override fun onPageFinished(view: WebView, url: String?) {
                    if (!finished) write()
                }

                override fun onReceivedError(
                    view: WebView,
                    request: android.webkit.WebResourceRequest,
                    error: android.webkit.WebResourceError,
                ) {
                    // Sub-resources (there should be none) must not fail the page.
                    if (request.isForMainFrame) {
                        finish(Result.failure(FlutterError("load-failed", error.description.toString(), null)))
                    }
                }

                override fun onRenderProcessGone(
                    view: WebView,
                    detail: android.webkit.RenderProcessGoneDetail,
                ): Boolean {
                    finish(Result.failure(FlutterError("web-process-ended", "The web view's process ended.", null)))
                    return true
                }
            }
        webView.loadDataWithBaseURL(null, html, "text/html", "utf-8", null)
    }

    private fun write() {
        val mils = (marginPt / 72.0 * 1000).toInt()
        val size =
            when (paper) {
                PrintPaper.A4 -> PrintAttributes.MediaSize.ISO_A4
                PrintPaper.LETTER -> PrintAttributes.MediaSize.NA_LETTER
            }
        val attributes =
            PrintAttributes.Builder()
                .setMediaSize(size)
                .setResolution(PrintAttributes.Resolution("kanso", "print", 300, 300))
                .setMinMargins(PrintAttributes.Margins(mils, mils, mils, mils))
                .setColorMode(PrintAttributes.COLOR_MODE_COLOR)
                .build()
        val file = File(outputPath)
        file.parentFile?.mkdirs()
        KansoPdfWriter.write(webView.createPrintDocumentAdapter("kanso"), attributes, file) { error ->
            if (error != null) {
                finish(Result.failure(FlutterError("print-failed", error.message ?: error.toString(), null)))
            } else {
                finish(pageCount(file))
            }
        }
    }

    private fun pageCount(file: File): Result<Int> =
        try {
            ParcelFileDescriptor.open(file, ParcelFileDescriptor.MODE_READ_ONLY).use { fd ->
                PdfRenderer(fd).use { Result.success(it.pageCount) }
            }
        } catch (e: Exception) {
            Result.failure(FlutterError("print-failed", "The PDF could not be read back: ${e.message}", null))
        }

    private fun finish(result: Result<Int>) {
        if (finished) return
        finished = true
        handler.removeCallbacks(timeout)
        webView.stopLoading()
        webView.destroy()
        done(result)
    }

    private companion object {
        const val TIMEOUT_MS = 60_000L
    }
}
