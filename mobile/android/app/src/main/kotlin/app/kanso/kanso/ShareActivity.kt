package app.kanso.kanso

import android.app.Activity
import android.content.ContentResolver
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.provider.OpenableColumns
import android.widget.Toast
import java.io.File
import org.json.JSONObject

/**
 * Takes what other apps share to Kanso (text, links, images, PDFs) and leaves
 * it in the Inbox without opening the app.
 *
 * The activity has no screen and starts no Flutter engine. It reads the
 * intent, writes one drop (see [DropWriter]) into `<filesDir>/kanso/inbox/` on
 * a background thread, asks the native library to save drops
 * ([KansoNative.captureInbox]), tells the person the outcome in a toast and
 * finishes. When the native call isn't available, the drop stays in the
 * folder and the app saves it the next time it opens.
 *
 * It doesn't ask the sender to keep access to a shared file: the files are
 * copied while the activity runs, and the access ends with it.
 */
class ShareActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val share = intent
        val appContext = applicationContext
        if (share == null) {
            finish()
            return
        }
        // The sender's access to its files lasts while this activity is
        // alive, so the copying is held by the activity's lifetime: finish
        // only when it is done.
        Thread {
            val message = save(share)
            runOnUiThread {
                Toast.makeText(appContext, message, Toast.LENGTH_LONG).show()
                finish()
            }
        }.start()
    }

    /** Writes the drop and saves it; returns the sentence to show. */
    private fun save(share: Intent): String {
        val library = File(filesDir, "kanso")
        val inbox = File(library, "inbox")
        val result = try {
            DropWriter(inbox).write(
                subject = share.getStringExtra(Intent.EXTRA_SUBJECT)
                    ?: share.getStringExtra(Intent.EXTRA_TITLE),
                text = share.getCharSequenceExtra(Intent.EXTRA_TEXT)?.toString(),
                files = sourcesOf(share),
            )
        } catch (_: Exception) {
            return getString(R.string.share_failed)
        }
        if (result.id == null) {
            return result.problems.joinToString(" ").ifEmpty { getString(R.string.share_nothing) }
        }
        val saved = try {
            KansoNative.captureInbox(File(library, "library.db").path, inbox.path)
        } catch (_: UnsatisfiedLinkError) {
            null
        } catch (_: Exception) {
            null
        } ?: return getString(R.string.share_kept)
        val problems = try {
            val list = JSONObject(saved).optJSONArray("problems")
            (0 until (list?.length() ?: 0)).map { list!!.getString(it) }
        } catch (_: Exception) {
            emptyList()
        }
        return (problems + result.problems).joinToString(" ").ifEmpty { getString(R.string.share_saved) }
    }

    /** The files in the intent (`EXTRA_STREAM`, one or a list), as sources the writer can stream. */
    private fun sourcesOf(share: Intent): List<DropSource> {
        val uris = linkedSetOf<Uri>()
        val stream = if (Build.VERSION.SDK_INT >= 33) {
            share.getParcelableExtra(Intent.EXTRA_STREAM, Uri::class.java)
        } else {
            @Suppress("DEPRECATION")
            share.getParcelableExtra<Uri>(Intent.EXTRA_STREAM)
        }
        stream?.let { uris += it }
        val list = if (Build.VERSION.SDK_INT >= 33) {
            share.getParcelableArrayListExtra(Intent.EXTRA_STREAM, Uri::class.java)
        } else {
            @Suppress("DEPRECATION")
            share.getParcelableArrayListExtra<Uri>(Intent.EXTRA_STREAM)
        }
        list?.let { uris += it.filterNotNull() }
        val resolver = contentResolver
        return uris.map { sourceOf(resolver, it, share.type) }
    }

    private fun sourceOf(resolver: ContentResolver, uri: Uri, sharedType: String?): DropSource {
        var name: String? = null
        var size: Long? = null
        try {
            resolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME, OpenableColumns.SIZE), null, null, null)
                ?.use { cursor ->
                    if (cursor.moveToFirst()) {
                        val nameColumn = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                        val sizeColumn = cursor.getColumnIndex(OpenableColumns.SIZE)
                        if (nameColumn >= 0 && !cursor.isNull(nameColumn)) {
                            name = cursor.getString(nameColumn)
                        }
                        if (sizeColumn >= 0 && !cursor.isNull(sizeColumn)) {
                            size = cursor.getLong(sizeColumn)
                        }
                    }
                }
        } catch (_: Exception) {
            // The sender gave no name or size; the file is still read below.
        }
        val type = try {
            resolver.getType(uri)
        } catch (_: Exception) {
            null
        } ?: sharedType?.takeIf { !it.endsWith("/*") }
        return DropSource(name, type, size) { resolver.openInputStream(uri) }
    }
}
