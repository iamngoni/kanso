package app.kanso.kanso_ocr

import org.junit.Assert.assertEquals
import org.junit.Test

class OcrMappingTest {
    @Test
    fun smallPicturesAreNotShrunk() {
        assertEquals(1, OcrMapping.sampleSize(800, 600))
        assertEquals(1, OcrMapping.sampleSize(4096, 4096))
    }

    @Test
    fun theLongestSideDecidesAndTheStepIsAPowerOfTwo() {
        assertEquals(2, OcrMapping.sampleSize(4097, 100))
        assertEquals(2, OcrMapping.sampleSize(100, 8192))
        assertEquals(2, OcrMapping.sampleSize(8193, 8193))
        assertEquals(4, OcrMapping.sampleSize(16385, 100))
        assertEquals(8, OcrMapping.sampleSize(30000, 100))
    }

    @Test
    fun linesJoinBlockByBlockInOrder() {
        val blocks = listOf(listOf("Title", "  subtitle "), listOf("", "body"), emptyList())
        assertEquals("Title\nsubtitle\nbody", OcrMapping.join(blocks))
    }

    @Test
    fun aPictureWithNoWordsIsEmpty() {
        assertEquals("", OcrMapping.join(emptyList()))
        assertEquals("", OcrMapping.join(listOf(listOf(" ", ""))))
    }

    @Test
    fun exifOrientationsMapToTurns() {
        assertEquals(0, OcrMapping.rotationDegrees(1))
        assertEquals(90, OcrMapping.rotationDegrees(6))
        assertEquals(180, OcrMapping.rotationDegrees(3))
        assertEquals(270, OcrMapping.rotationDegrees(8))
        assertEquals(0, OcrMapping.rotationDegrees(0))
    }
}
