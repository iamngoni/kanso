package app.kanso.kanso

import android.content.ContentProvider
import android.content.ContentValues
import android.database.Cursor
import android.database.MatrixCursor
import android.net.Uri
import android.os.ParcelFileDescriptor
import android.provider.OpenableColumns
import java.io.File

/**
 * Serves the capture tests' files: `content://<authority>/<asset name>` is the
 * test asset of that name, with its name, size and type, as a sharing app's
 * provider gives them. Only the share's grant opens it for Kanso.
 */
class CaptureProvider : ContentProvider() {
    override fun onCreate() = true

    override fun getType(uri: Uri): String? = when (uri.lastPathSegment?.substringAfterLast('.')) {
        "png" -> "image/png"
        "pdf" -> "application/pdf"
        else -> null
    }

    override fun query(
        uri: Uri,
        projection: Array<String>?,
        selection: String?,
        selectionArgs: Array<String>?,
        sortOrder: String?,
    ): Cursor {
        val name = uri.lastPathSegment!!
        val cursor = MatrixCursor(arrayOf(OpenableColumns.DISPLAY_NAME, OpenableColumns.SIZE))
        cursor.addRow(arrayOf<Any>(name, copy(name).length()))
        return cursor
    }

    override fun openFile(uri: Uri, mode: String): ParcelFileDescriptor =
        ParcelFileDescriptor.open(copy(uri.lastPathSegment!!), ParcelFileDescriptor.MODE_READ_ONLY)

    /** The asset as a file in the cache, which a descriptor can be opened on. */
    private fun copy(name: String): File {
        val file = File(context!!.cacheDir, "capture-$name")
        context!!.assets.open(name).use { input ->
            file.outputStream().use { input.copyTo(it) }
        }
        return file
    }

    override fun insert(uri: Uri, values: ContentValues?): Uri? = null

    override fun delete(uri: Uri, selection: String?, selectionArgs: Array<String>?) = 0

    override fun update(uri: Uri, values: ContentValues?, selection: String?, selectionArgs: Array<String>?) = 0
}
