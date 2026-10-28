package app.kanso.kanso_ocr

import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.Typeface
import androidx.test.ext.junit.runners.AndroidJUnit4
import java.io.ByteArrayOutputStream
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class OcrEngineTest {
    /** [text] in large black type on white, as PNG bytes. */
    private fun picture(text: String): ByteArray {
        val bitmap = Bitmap.createBitmap(800, 300, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bitmap)
        canvas.drawColor(Color.WHITE)
        val paint =
            Paint(Paint.ANTI_ALIAS_FLAG).apply {
                color = Color.BLACK
                textSize = 160f
                typeface = Typeface.DEFAULT_BOLD
                textAlign = Paint.Align.CENTER
            }
        canvas.drawText(text, 400f, 200f, paint)
        val out = ByteArrayOutputStream()
        bitmap.compress(Bitmap.CompressFormat.PNG, 100, out)
        return out.toByteArray()
    }

    @Test
    fun readsKanso() =
        runBlocking {
            val engine = OcrEngine()
            assertTrue(engine.isAvailable())
            assertEquals("kanso", engine.read(picture("KANSO")).lowercase())
            engine.close()
        }

    @Test
    fun aBlankPictureHasNoWords() =
        runBlocking {
            val engine = OcrEngine()
            assertEquals("", engine.read(picture("")))
            engine.close()
        }

    @Test
    fun bytesThatAreNoImageAreUnreadable() =
        runBlocking {
            val engine = OcrEngine()
            try {
                engine.read(byteArrayOf(1, 2, 3, 4))
                fail("expected UnreadableImageException")
            } catch (_: UnreadableImageException) {
            }
            engine.close()
        }

    @Test
    fun theEngineIsNamedMlkit() {
        assertEquals("mlkit", KansoOcrPlugin().engine())
    }
}
