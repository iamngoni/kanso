package app.kanso.kanso_lock

/**
 * Hands an answer to Rust: `KansoNative.biometricAnswer` in the app module,
 * which this plugin cannot see. `KansoApplication.onCreate` installs it.
 *
 * The answer holds the biometric secret, so it goes from here to Rust and is
 * never returned to Dart, logged or kept.
 */
fun interface BiometricAnswerEntry {
    /**
     * 1 when a check took the answer, 0 when none waits, -1 when the secret
     * did not open the kept keys, -5 on a fault.
     */
    fun answer(
        ticket: Long,
        status: Int,
        secret: ByteArray?,
    ): Int
}

object RustAnswer {
    @Volatile
    private var entry: BiometricAnswerEntry? = null

    /** Installs the call into the library. */
    @JvmStatic
    fun install(entry: BiometricAnswerEntry) {
        this.entry = entry
    }

    /** Gives the answer to the check waiting on [ticket]; Rust's reply, -5 on a fault. */
    fun deliver(
        ticket: Long,
        status: BiometricStatus,
        secret: ByteArray? = null,
    ): Int =
        try {
            entry?.answer(ticket, status.code, secret) ?: -5
        } catch (error: Throwable) {
            -5
        }
}
