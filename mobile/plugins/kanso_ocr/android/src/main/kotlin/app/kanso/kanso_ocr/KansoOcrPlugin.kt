// Kanso — KansoOcrPlugin.kt
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

package app.kanso.kanso_ocr

import io.flutter.embedding.engine.plugins.FlutterPlugin

/**
 * The platform's text recognizer, Google ML Kit's bundled Latin model. The
 * picture never leaves the device and no permission is needed.
 */
class KansoOcrPlugin :
    FlutterPlugin,
    OcrHostApi {
    private val ocr = OcrEngine()

    override fun onAttachedToEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        OcrHostApi.setUp(binding.binaryMessenger, this)
    }

    override fun onDetachedFromEngine(binding: FlutterPlugin.FlutterPluginBinding) {
        OcrHostApi.setUp(binding.binaryMessenger, null)
        ocr.close()
    }

    override suspend fun read(image: ByteArray): String =
        try {
            ocr.read(image)
        } catch (_: UnreadableImageException) {
            throw FlutterError("unreadable", "The bytes are not an image this device can decode.", null)
        } catch (e: Exception) {
            throw FlutterError("failed", e.message ?: "Text recognition failed.", null)
        }

    override fun isAvailable(): Boolean = ocr.isAvailable()

    override fun engine(): String = ENGINE

    companion object {
        /** The name written beside the text. */
        const val ENGINE = "mlkit"
    }
}
