// Kanso — OcrMapping.kt
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

package app.kanso.kanso_ocr

/** The parts of reading a picture that need no Android: they run in plain JVM tests. */
object OcrMapping {
    /** The longest side, in pixels, a picture is read at; bigger ones are shrunk. */
    const val MAX_SIDE = 4096

    /**
     * The power-of-two `inSampleSize` that brings the longest side of a
     * [width] by [height] picture to [maxSide] or less (1 when it already is).
     */
    fun sampleSize(width: Int, height: Int, maxSide: Int = MAX_SIDE): Int {
        var sample = 1
        val longest = maxOf(width, height)
        while (longest / sample > maxSide) sample *= 2
        return sample
    }

    /**
     * The words of the recognizer's blocks, each a list of its lines in
     * order: one line per line, blocks one after the other, blank lines
     * dropped.
     */
    fun join(blocks: List<List<String>>): String =
        blocks
            .flatten()
            .map { it.trim() }
            .filter { it.isNotEmpty() }
            .joinToString("\n")

    /** The clockwise degrees an EXIF orientation tag asks to turn the picture by. */
    fun rotationDegrees(exifOrientation: Int): Int =
        when (exifOrientation) {
            6, 5 -> 90 // ROTATE_90, TRANSPOSE
            3, 4 -> 180 // ROTATE_180, FLIP_VERTICAL
            8, 7 -> 270 // ROTATE_270, TRANSVERSE
            else -> 0
        }
}
