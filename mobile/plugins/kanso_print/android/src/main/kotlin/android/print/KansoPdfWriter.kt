package android.print

import android.os.CancellationSignal
import android.os.ParcelFileDescriptor
import java.io.File

/**
 * Drives a [PrintDocumentAdapter] to a PDF file with no print dialog.
 *
 * It sits in `android.print` because the layout and write callbacks have
 * package-private constructors. Call it on the main thread; [done] gets
 * `null` on success or the failure, once.
 */
object KansoPdfWriter {
    fun write(
        adapter: PrintDocumentAdapter,
        attributes: PrintAttributes,
        file: File,
        done: (Throwable?) -> Unit,
    ) {
        var reported = false

        fun report(error: Throwable?) {
            if (reported) return
            reported = true
            done(error)
        }

        try {
            adapter.onStart()
            adapter.onLayout(
                null,
                attributes,
                CancellationSignal(),
                object : PrintDocumentAdapter.LayoutResultCallback() {
                    override fun onLayoutFinished(info: PrintDocumentInfo?, changed: Boolean) {
                        try {
                            val fd =
                                ParcelFileDescriptor.open(
                                    file,
                                    ParcelFileDescriptor.MODE_CREATE or
                                        ParcelFileDescriptor.MODE_TRUNCATE or
                                        ParcelFileDescriptor.MODE_READ_WRITE,
                                )
                            adapter.onWrite(
                                arrayOf(PageRange.ALL_PAGES),
                                fd,
                                CancellationSignal(),
                                object : PrintDocumentAdapter.WriteResultCallback() {
                                    override fun onWriteFinished(pages: Array<out PageRange>?) {
                                        fd.close()
                                        adapter.onFinish()
                                        report(null)
                                    }

                                    override fun onWriteFailed(error: CharSequence?) {
                                        fd.close()
                                        adapter.onFinish()
                                        report(RuntimeException(error?.toString() ?: "The write failed."))
                                    }

                                    override fun onWriteCancelled() {
                                        fd.close()
                                        adapter.onFinish()
                                        report(RuntimeException("The write was cancelled."))
                                    }
                                },
                            )
                        } catch (e: Throwable) {
                            report(e)
                        }
                    }

                    override fun onLayoutFailed(error: CharSequence?) {
                        report(RuntimeException(error?.toString() ?: "The layout failed."))
                    }

                    override fun onLayoutCancelled() {
                        report(RuntimeException("The layout was cancelled."))
                    }
                },
                null,
            )
        } catch (e: Throwable) {
            report(e)
        }
    }
}
