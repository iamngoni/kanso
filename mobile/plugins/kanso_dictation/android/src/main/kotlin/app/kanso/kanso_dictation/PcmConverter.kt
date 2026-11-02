package app.kanso.kanso_dictation

import kotlin.math.floor

/**
 * Turns decoded PCM (16-bit, any channel count, any rate) into 16 kHz mono
 * 16-bit little-endian PCM, which is what the platform recognizer takes from
 * a file (`EXTRA_AUDIO_SOURCE`). It touches no Android API.
 *
 * Feed the decoder's output in order with [convert]; it keeps the little
 * state it needs between calls (the last sample and the position of the next
 * output sample), so the pieces can be any size.
 *
 * @param sourceRate the decoder's sample rate.
 * @param channels the decoder's channel count; channels are averaged.
 */
class PcmConverter(private val sourceRate: Int, private val channels: Int) {
    companion object {
        /** The rate the recognizer is given. */
        const val TARGET_RATE = 16_000

        /** The mean of one interleaved frame. */
        fun downmix(interleaved: ShortArray, channels: Int): ShortArray {
            if (channels <= 1) {
                return interleaved
            }
            val frames = interleaved.size / channels
            return ShortArray(frames) { frame ->
                var sum = 0
                for (c in 0 until channels) {
                    sum += interleaved[frame * channels + c]
                }
                (sum / channels).toShort()
            }
        }

        /** [samples] as little-endian bytes. */
        fun toBytes(samples: ShortArray): ByteArray {
            val out = ByteArray(samples.size * 2)
            for (i in samples.indices) {
                out[i * 2] = (samples[i].toInt() and 0xff).toByte()
                out[i * 2 + 1] = (samples[i].toInt() shr 8).toByte()
            }
            return out
        }
    }

    private val step = sourceRate.toDouble() / TARGET_RATE
    private var next = 0.0 // input index of the next output sample
    private var consumed = 0L // input samples before the current piece
    private var last = 0.toShort() // the sample just before the current piece

    /** Converts one more [interleaved] piece of the decoder's output. */
    fun convert(interleaved: ShortArray): ShortArray {
        val mono = downmix(interleaved, channels)
        if (mono.isEmpty()) {
            return ShortArray(0)
        }
        if (sourceRate == TARGET_RATE) {
            consumed += mono.size
            return mono
        }
        val end = consumed + mono.size - 1 // the piece's last input index
        val out = ArrayList<Short>((mono.size / step).toInt() + 1)
        while (true) {
            val i0 = floor(next).toLong()
            if (i0 + 1 > end) {
                break
            }
            val a = sample(mono, i0)
            val b = sample(mono, i0 + 1)
            val fraction = next - i0
            out += (a + (b - a) * fraction).toInt().toShort()
            next += step
        }
        last = mono[mono.size - 1]
        consumed += mono.size
        return out.toShortArray()
    }

    private fun sample(piece: ShortArray, index: Long): Int =
        if (index < consumed) last.toInt() else piece[(index - consumed).toInt()].toInt()
}
