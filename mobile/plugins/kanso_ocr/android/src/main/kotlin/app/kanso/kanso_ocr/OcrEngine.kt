// Kanso — OcrEngine.kt
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

package app.kanso.kanso_ocr

import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.ImageDecoder
import android.os.Build
import androidx.exifinterface.media.ExifInterface
import com.google.mlkit.vision.common.InputImage
import com.google.mlkit.vision.text.TextRecognition
import com.google.mlkit.vision.text.TextRecognizer
import com.google.mlkit.vision.text.latin.TextRecognizerOptions
import java.io.ByteArrayInputStream
import java.nio.ByteBuffer
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withContext
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException

/** The bytes are no picture this device can decode. */
class UnreadableImageException : Exception("The bytes are not an image this device can decode.")

/**
 * Google ML Kit's bundled Latin text recognizer. The model ships inside the
 * app, so it works offline with nothing to download, and the picture never
 * leaves the device.
 */
class OcrEngine {
    private var client: TextRecognizer? = null

    /** The client, made on first use; null when ML Kit cannot make one. */
    private fun recognizer(): TextRecognizer? {
        client?.let { return it }
        return try {
            TextRecognition.getClient(TextRecognizerOptions.DEFAULT_OPTIONS).also { client = it }
        } catch (_: Throwable) {
            null
        }
    }

    /** Whether the recognizer client can be created. */
    fun isAvailable(): Boolean = recognizer() != null

    /**
     * The words in [bytes] (PNG, JPEG, WebP, and HEIC/HEIF where the device
     * decodes it), one line per line; empty when there are none. Throws
     * [UnreadableImageException] for bytes that are no picture.
     */
    suspend fun read(bytes: ByteArray): String {
        val recognizer = recognizer() ?: throw IllegalStateException("ML Kit is not available.")
        val (bitmap, rotation) = withContext(Dispatchers.Default) { decode(bytes) }
        try {
            val text = recognize(recognizer, InputImage.fromBitmap(bitmap, rotation))
            return OcrMapping.join(text)
        } finally {
            bitmap.recycle()
        }
    }

    fun close() {
        client?.close()
        client = null
    }

    private suspend fun recognize(recognizer: TextRecognizer, image: InputImage): List<List<String>> =
        suspendCancellableCoroutine { continuation ->
            recognizer
                .process(image)
                .addOnSuccessListener { text ->
                    continuation.resume(text.textBlocks.map { block -> block.lines.map { it.text } })
                }.addOnFailureListener { continuation.resumeWithException(it) }
        }

    /** The picture, shrunk to [OcrMapping.MAX_SIDE], and the turn it needs. */
    private fun decode(bytes: ByteArray): Pair<Bitmap, Int> {
        try {
            val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
            BitmapFactory.decodeByteArray(bytes, 0, bytes.size, bounds)
            if (bounds.outWidth > 0 && bounds.outHeight > 0) {
                val options =
                    BitmapFactory.Options().apply {
                        inSampleSize = OcrMapping.sampleSize(bounds.outWidth, bounds.outHeight)
                    }
                BitmapFactory.decodeByteArray(bytes, 0, bytes.size, options)?.let {
                    return it to rotation(bytes)
                }
            }
            // HEIC/HEIF that BitmapFactory leaves alone (API 28 and 29).
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                val bitmap =
                    ImageDecoder.decodeBitmap(ImageDecoder.createSource(ByteBuffer.wrap(bytes))) { decoder, info, _ ->
                        decoder.allocator = ImageDecoder.ALLOCATOR_SOFTWARE
                        val sample = OcrMapping.sampleSize(info.size.width, info.size.height)
                        if (sample > 1) decoder.setTargetSampleSize(sample)
                    }
                // ImageDecoder turns the picture upright itself.
                return bitmap to 0
            }
        } catch (_: Exception) {
        } catch (_: OutOfMemoryError) {
        }
        throw UnreadableImageException()
    }

    private fun rotation(bytes: ByteArray): Int =
        try {
            val orientation =
                ExifInterface(ByteArrayInputStream(bytes))
                    .getAttributeInt(ExifInterface.TAG_ORIENTATION, ExifInterface.ORIENTATION_NORMAL)
            OcrMapping.rotationDegrees(orientation)
        } catch (_: Exception) {
            0
        }
}
