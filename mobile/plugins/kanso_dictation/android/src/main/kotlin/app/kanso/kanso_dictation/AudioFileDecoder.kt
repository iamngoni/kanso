package app.kanso.kanso_dictation

import android.media.MediaCodec
import android.media.MediaExtractor
import android.media.MediaFormat
import java.io.File
import java.io.IOException
import java.nio.ByteOrder

/** What [AudioFileDecoder.decode] made: the PCM file's length in milliseconds. */
class DecodedAudio(val durationMs: Long)

/**
 * Decodes an audio file (the voice drop's `rec.m4a`) to 16 kHz mono 16-bit
 * PCM in [target], with `MediaExtractor` and `MediaCodec`. Streams, so a
 * 30-minute recording is never held in memory.
 */
object AudioFileDecoder {
    private const val TIMEOUT_US = 10_000L

    /**
     * @throws IOException when the file isn't audio the device can decode.
     */
    fun decode(source: File, target: File): DecodedAudio {
        val extractor = MediaExtractor()
        var codec: MediaCodec? = null
        try {
            try {
                extractor.setDataSource(source.path)
            } catch (error: Exception) {
                throw IOException("The audio can't be opened.", error)
            }
            val index = (0 until extractor.trackCount).firstOrNull {
                extractor.getTrackFormat(it).getString(MediaFormat.KEY_MIME)?.startsWith("audio/") == true
            } ?: throw IOException("The file has no audio.")
            extractor.selectTrack(index)
            val format = extractor.getTrackFormat(index)
            val mime = format.getString(MediaFormat.KEY_MIME) ?: throw IOException("The file has no audio.")
            codec = try {
                MediaCodec.createDecoderByType(mime).also {
                    it.configure(format, null, null, 0)
                    it.start()
                }
            } catch (error: Exception) {
                throw IOException("The audio can't be decoded.", error)
            }
            var rate = format.getInteger(MediaFormat.KEY_SAMPLE_RATE)
            var channels = format.getInteger(MediaFormat.KEY_CHANNEL_COUNT)
            var converter = PcmConverter(rate, channels)
            var written = 0L
            val info = MediaCodec.BufferInfo()
            var inputDone = false
            var outputDone = false
            target.outputStream().buffered().use { out ->
                while (!outputDone) {
                    if (!inputDone) {
                        val slot = codec.dequeueInputBuffer(TIMEOUT_US)
                        if (slot >= 0) {
                            val buffer = codec.getInputBuffer(slot)!!
                            val size = extractor.readSampleData(buffer, 0)
                            if (size < 0) {
                                codec.queueInputBuffer(slot, 0, 0, 0, MediaCodec.BUFFER_FLAG_END_OF_STREAM)
                                inputDone = true
                            } else {
                                codec.queueInputBuffer(slot, 0, size, extractor.sampleTime, 0)
                                extractor.advance()
                            }
                        }
                    }
                    val slot = codec.dequeueOutputBuffer(info, TIMEOUT_US)
                    when {
                        slot == MediaCodec.INFO_OUTPUT_FORMAT_CHANGED -> {
                            val changed = codec.outputFormat
                            val newRate = changed.getInteger(MediaFormat.KEY_SAMPLE_RATE)
                            val newChannels = changed.getInteger(MediaFormat.KEY_CHANNEL_COUNT)
                            if (newRate != rate || newChannels != channels) {
                                rate = newRate
                                channels = newChannels
                                converter = PcmConverter(rate, channels)
                            }
                        }
                        slot >= 0 -> {
                            val buffer = codec.getOutputBuffer(slot)!!
                            buffer.position(info.offset)
                            buffer.limit(info.offset + info.size)
                            val shorts = ShortArray(info.size / 2)
                            buffer.order(ByteOrder.LITTLE_ENDIAN).asShortBuffer().get(shorts)
                            val bytes = PcmConverter.toBytes(converter.convert(shorts))
                            out.write(bytes)
                            written += bytes.size
                            codec.releaseOutputBuffer(slot, false)
                            if (info.flags and MediaCodec.BUFFER_FLAG_END_OF_STREAM != 0) {
                                outputDone = true
                            }
                        }
                    }
                }
            }
            return DecodedAudio(written / 2 * 1000 / PcmConverter.TARGET_RATE)
        } catch (error: IOException) {
            target.delete()
            throw error
        } catch (error: RuntimeException) {
            target.delete()
            throw IOException("The audio can't be decoded.", error)
        } finally {
            try {
                codec?.stop()
            } catch (_: RuntimeException) {
            }
            codec?.release()
            extractor.release()
        }
    }
}
