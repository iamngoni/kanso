// Kanso: KansoFiles.kt
// Licensed under AGPL-3.0-or-later (see LICENSE).

package app.kanso.kanso

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.Handler
import android.os.Looper
import android.provider.DocumentsContract
import android.provider.OpenableColumns
import android.webkit.MimeTypeMap
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import java.io.File
import java.util.UUID

/**
 * Files in and out of Kanso on Android, behind the `app.kanso/files` channel.
 *
 * Importing: the system's document picker chooses a file or a folder and this
 * copies the choice under `cache/import/<name>`, which is where the Dart side
 * (`mobile/lib/import/`) and the Rust core read it, since a content address is
 * not a path. The copy is removed with `clearPicked`.
 *
 * Opening: Markdown and text files other apps open in Kanso (the manifest's
 * `VIEW` intent filters) reach [handleIntent]. Each is copied under
 * `cache/opened/<n>/<name>` with its content address as its identity, queued,
 * and announced to Dart with `opened`. Dart takes the queue with `takeOpened`
 * and removes the copies with `discardOpened`; a document that arrives before
 * Dart listens waits in the queue. Nothing is decoded here.
 */
class KansoFiles(private val activity: Activity) {
    private var channel: MethodChannel? = null
    private var pending: MethodChannel.Result? = null
    private var pendingFolder = false
    private val queued = mutableListOf<Map<String, String>>()
    private val taken = mutableListOf<File>()
    private val main = Handler(Looper.getMainLooper())

    /** Answers the `app.kanso/files` channel. */
    fun register(engine: FlutterEngine) {
        val channel = MethodChannel(engine.dartExecutor.binaryMessenger, CHANNEL)
        channel.setMethodCallHandler { call, result -> handle(call, result) }
        this.channel = channel
        if (synchronized(queued) { queued.isNotEmpty() }) {
            channel.invokeMethod("opened", null)
        }
    }

    private fun handle(call: MethodCall, result: MethodChannel.Result) {
        when (call.method) {
            "pickFile" -> {
                val extensions = call.argument<List<String>>("extensions") ?: emptyList()
                pickFile(extensions, result)
            }
            "pickFolder" -> pickFolder(result)
            "clearPicked" -> {
                folder("import").deleteRecursively()
                result.success(null)
            }
            "takeOpened" -> {
                val files = synchronized(queued) {
                    val batch = queued.toList()
                    queued.clear()
                    batch
                }
                for (entry in files) {
                    entry["path"]?.let { taken.add(File(it).parentFile ?: File(it)) }
                }
                result.success(
                    mapOf("staging" to folder("opened-staging").path, "files" to files),
                )
            }
            "discardOpened" -> {
                taken.forEach { it.deleteRecursively() }
                taken.clear()
                result.success(null)
            }
            else -> result.notImplemented()
        }
    }

    private fun folder(name: String): File = File(activity.cacheDir, name)

    // Picking

    private fun pickFile(extensions: List<String>, result: MethodChannel.Result) {
        if (!claim(result, folder = false)) {
            return
        }
        val types = extensions.mapNotNull {
            MimeTypeMap.getSingleton().getMimeTypeFromExtension(it.lowercase())
        }
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*"
            // Narrowed to the source's types when every extension has one the
            // system knows; any file otherwise, since the importer says what
            // a file is.
            if (types.isNotEmpty() && types.size == extensions.size) {
                putExtra(Intent.EXTRA_MIME_TYPES, (types + "application/octet-stream").toTypedArray())
            }
        }
        activity.startActivityForResult(intent, PICK_FILE)
    }

    private fun pickFolder(result: MethodChannel.Result) {
        if (!claim(result, folder = true)) {
            return
        }
        activity.startActivityForResult(Intent(Intent.ACTION_OPEN_DOCUMENT_TREE), PICK_FOLDER)
    }

    private fun claim(result: MethodChannel.Result, folder: Boolean): Boolean {
        if (pending != null) {
            result.error("busy", "A picker is already open.", null)
            return false
        }
        pending = result
        pendingFolder = folder
        return true
    }

    /** Call from `Activity.onActivityResult`; whether the result was the picker's. */
    fun handleResult(requestCode: Int, resultCode: Int, data: Intent?): Boolean {
        if (requestCode != PICK_FILE && requestCode != PICK_FOLDER) {
            return false
        }
        val result = pending ?: return true
        pending = null
        val isFolder = pendingFolder
        val uri = data?.data
        if (resultCode != Activity.RESULT_OK || uri == null) {
            result.success(null)
            return true
        }
        Thread {
            try {
                val copy = if (isFolder) copyFolder(uri) else copyFile(uri)
                main.post { result.success(mapOf("path" to copy.path, "name" to copy.name)) }
            } catch (error: Exception) {
                main.post { result.error("copy_failed", error.message, null) }
            }
        }.start()
        return true
    }

    private fun copyFile(uri: Uri): File {
        val target = File(folder("import").apply { deleteRecursively(); mkdirs() }, nameOf(uri))
        copyStream(uri, target)
        return target
    }

    private fun copyFolder(tree: Uri): File {
        val root = DocumentsContract.getTreeDocumentId(tree)
        val rootUri = DocumentsContract.buildDocumentUriUsingTree(tree, root)
        val target = File(folder("import").apply { deleteRecursively(); mkdirs() }, nameOf(rootUri))
        target.mkdirs()
        copyChildren(tree, root, target)
        return target
    }

    private fun copyChildren(tree: Uri, parent: String, into: File) {
        val children = DocumentsContract.buildChildDocumentsUriUsingTree(tree, parent)
        val columns = arrayOf(
            DocumentsContract.Document.COLUMN_DOCUMENT_ID,
            DocumentsContract.Document.COLUMN_DISPLAY_NAME,
            DocumentsContract.Document.COLUMN_MIME_TYPE,
        )
        activity.contentResolver.query(children, columns, null, null, null)?.use { cursor ->
            while (cursor.moveToNext()) {
                val id = cursor.getString(0)
                val name = safeName(cursor.getString(1))
                val target = File(into, name)
                if (cursor.getString(2) == DocumentsContract.Document.MIME_TYPE_DIR) {
                    target.mkdirs()
                    copyChildren(tree, id, target)
                } else {
                    copyStream(DocumentsContract.buildDocumentUriUsingTree(tree, id), target)
                }
            }
        }
    }

    private fun copyStream(uri: Uri, target: File) {
        val input = activity.contentResolver.openInputStream(uri)
            ?: throw java.io.IOException("Couldn't read $target")
        input.use { source -> target.outputStream().use { source.copyTo(it) } }
    }

    private fun nameOf(uri: Uri): String {
        val name = activity.contentResolver
            .query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)
            ?.use { if (it.moveToFirst()) it.getString(0) else null }
        return safeName(name ?: uri.lastPathSegment)
    }

    /** A name that can't leave the folder it is made in. */
    private fun safeName(name: String?): String {
        val clean = (name ?: "").substringAfterLast('/').trim()
        return if (clean.isEmpty() || clean == "." || clean == "..") "imported" else clean
    }

    // Opening

    /**
     * Call with the launch intent and each new one: queues the document a
     * `VIEW` intent opens. Kanso's own links (`kanso://`) are not documents
     * and are left to the router.
     */
    fun handleIntent(intent: Intent?) {
        val uri = intent?.data ?: return
        if (intent.action != Intent.ACTION_VIEW || (uri.scheme != "content" && uri.scheme != "file")) {
            return
        }
        Thread {
            try {
                val folder = File(folder("opened"), UUID.randomUUID().toString()).apply { mkdirs() }
                val target = File(folder, nameOf(uri))
                copyStream(uri, target)
                synchronized(queued) {
                    queued.add(
                        mapOf("path" to target.path, "name" to target.name, "identity" to uri.toString()),
                    )
                }
                main.post { channel?.invokeMethod("opened", null) }
            } catch (_: Exception) {
                val failed = File(folder("opened"), UUID.randomUUID().toString())
                val name = safeName(uri.lastPathSegment)
                synchronized(queued) {
                    queued.add(
                        mapOf(
                            "path" to File(failed, name).path,
                            "name" to name,
                            "identity" to uri.toString(),
                        ),
                    )
                }
                main.post { channel?.invokeMethod("opened", null) }
            }
        }.start()
    }

    companion object {
        /** The method channel's name, the same on both platforms. */
        const val CHANNEL = "app.kanso/files"

        private const val PICK_FILE = 4801
        private const val PICK_FOLDER = 4802
    }
}
