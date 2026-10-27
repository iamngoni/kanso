package app.kanso.kanso

import java.io.File
import java.io.IOException
import java.io.InputStream
import java.security.SecureRandom
import org.json.JSONArray
import org.json.JSONObject

/**
 * One file offered by a share, as the writer needs it: what the sharing app
 * called it, the type it gave, its size when known, and a way to read it.
 *
 * [open] is called at most once and may return `null` when the file can't be
 * read. The writer closes the stream.
 */
class DropSource(
    val name: String?,
    val type: String?,
    val size: Long?,
    val open: () -> InputStream?,
)

/** What [DropWriter.write] did: the new drop's [id] (`null` when nothing was kept) and its [problems], in plain words. */
class DropResult(val id: String?, val problems: List<String>)

/**
 * Writes a share as a *drop*, a folder in the library's `inbox/`
 * (`mobile/docs/capture.md`).
 *
 * The folder is built as `<id>.partial`: the files first, under names of the
 * writer's own (`0.png`, `1.pdf`), then `drop.json`, then it is renamed to
 * `<id>`. A reader never opens a `.partial` folder, so it never sees half a
 * drop. Files are streamed in small pieces, never held whole in memory.
 *
 * This class touches no Android API, so a plain JVM test can run it.
 *
 * @param inbox the library's `inbox/` folder; made if missing.
 * @param random the source of the drop's id.
 * @param now the clock, in milliseconds since 1970.
 */
class DropWriter(
    private val inbox: File,
    private val random: SecureRandom = SecureRandom(),
    private val now: () -> Long = System::currentTimeMillis,
) {
    companion object {
        /** The most files one drop holds. */
        const val MAX_FILES = 20

        /** The largest file one drop takes (the library's attachment limit), 10 MB. */
        const val MAX_FILE_BYTES = 10L * 1024 * 1024

        private const val ID_BYTES = 16
        private val URL = Regex("""https?://\S+""")

        /** A fresh id: 32 lowercase hex characters from [random]. */
        fun newId(random: SecureRandom): String {
            val bytes = ByteArray(ID_BYTES)
            random.nextBytes(bytes)
            return bytes.joinToString("") { "%02x".format(it) }
        }

        /** The extension (without the dot, lowercase, letters and digits only) for a file called [name] of type [type]. */
        fun extensionOf(name: String?, type: String?): String {
            val fromName = name
                ?.substringAfterLast('/')
                ?.substringAfterLast('.', "")
                ?.lowercase()
                ?.takeIf { it.length in 1..8 && it.all { c -> c in 'a'..'z' || c in '0'..'9' } }
            if (fromName != null) {
                return fromName
            }
            return when (type?.lowercase()?.substringBefore(';')?.trim()) {
                "image/png" -> "png"
                "image/jpeg", "image/jpg" -> "jpg"
                "image/gif" -> "gif"
                "image/webp" -> "webp"
                "image/heic" -> "heic"
                "image/heif" -> "heif"
                "application/pdf" -> "pdf"
                else -> "bin"
            }
        }

        /** The first web link in [text], or `null`. */
        fun urlIn(text: String?): String? = text?.let { URL.find(it)?.value }
    }

    /**
     * Writes one drop of [subject], [text] and [files] and returns its id and
     * the problems met on the way. A file that is too large, unreadable or
     * beyond the [MAX_FILES]th is left out and named in the problems. When no
     * text and no file is left, no drop is made and the id is `null`.
     *
     * @throws IOException when the folder can't be written.
     */
    fun write(subject: String?, text: String?, files: List<DropSource>): DropResult {
        if (!inbox.isDirectory && !inbox.mkdirs()) {
            throw IOException("The inbox folder can't be made.")
        }
        val id = newId(random)
        val partial = File(inbox, "$id.partial")
        if (!partial.mkdir()) {
            throw IOException("The drop's folder can't be made.")
        }
        try {
            val problems = mutableListOf<String>()
            val kept = JSONArray()
            for (source in files) {
                val label = source.name?.takeIf { it.isNotBlank() } ?: "A file"
                if (kept.length() >= MAX_FILES) {
                    problems += "Only the first $MAX_FILES files are kept; $label was left out."
                    continue
                }
                if (source.size != null && source.size > MAX_FILE_BYTES) {
                    problems += "$label is larger than 10 MB, so it was left out."
                    continue
                }
                val path = "${kept.length()}.${extensionOf(source.name, source.type)}"
                val target = File(partial, path)
                when (copy(source, target)) {
                    Copy.DONE -> kept.put(
                        JSONObject()
                            .put("path", path)
                            .apply {
                                source.name?.let { put("name", it) }
                                source.type?.let { put("type", it) }
                            },
                    )
                    Copy.TOO_LARGE -> problems += "$label is larger than 10 MB, so it was left out."
                    Copy.UNREADABLE -> problems += "$label couldn't be read, so it was left out."
                }
            }
            val cleanText = text?.takeIf { it.isNotBlank() }
            if (cleanText == null && kept.length() == 0) {
                partial.deleteRecursively()
                return DropResult(null, problems)
            }
            val json = JSONObject().put("version", 1).put("createdAt", now())
            subject?.takeIf { it.isNotBlank() }?.let { json.put("subject", it) }
            cleanText?.let {
                json.put("text", it)
                urlIn(it)?.let { link -> json.put("url", link) }
            }
            if (kept.length() > 0) {
                json.put("files", kept)
            }
            File(partial, "drop.json").writeText(json.toString(), Charsets.UTF_8)
            if (!partial.renameTo(File(inbox, id))) {
                throw IOException("The drop can't be finished.")
            }
            return DropResult(id, problems)
        } catch (error: Throwable) {
            partial.deleteRecursively()
            throw error
        }
    }

    private enum class Copy { DONE, TOO_LARGE, UNREADABLE }

    /** Streams [source] into [target], stopping at [MAX_FILE_BYTES]; removes [target] unless it is whole. */
    private fun copy(source: DropSource, target: File): Copy {
        var done = false
        try {
            val input = try {
                source.open()
            } catch (_: IOException) {
                null
            } catch (_: SecurityException) {
                null
            } ?: return Copy.UNREADABLE
            input.use { stream ->
                target.outputStream().use { out ->
                    val buffer = ByteArray(32 * 1024)
                    var total = 0L
                    while (true) {
                        val count = stream.read(buffer)
                        if (count < 0) {
                            break
                        }
                        total += count
                        if (total > MAX_FILE_BYTES) {
                            return Copy.TOO_LARGE
                        }
                        out.write(buffer, 0, count)
                    }
                }
            }
            done = true
            return Copy.DONE
        } catch (_: IOException) {
            return Copy.UNREADABLE
        } finally {
            if (!done) {
                target.delete()
            }
        }
    }
}
