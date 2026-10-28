package app.kanso.kanso_lock

import androidx.biometric.BiometricPrompt

/**
 * The status codes of `KansoNative.biometricAnswer` and what Dart is told.
 * The mapping is pure so it can be tested without a device.
 */
enum class BiometricStatus(
    val code: Int,
    val outcome: BiometricOutcome,
) {
    OK(0, BiometricOutcome.DONE),
    CANCELLED(1, BiometricOutcome.CANCELLED),
    INVALIDATED(2, BiometricOutcome.INVALIDATED),
    UNAVAILABLE(3, BiometricOutcome.UNAVAILABLE),
    FAILED(4, BiometricOutcome.FAILED),
    ;

    companion object {
        /** The status of a `BiometricPrompt` error code. */
        fun ofPromptError(code: Int): BiometricStatus =
            when (code) {
                BiometricPrompt.ERROR_USER_CANCELED,
                BiometricPrompt.ERROR_NEGATIVE_BUTTON,
                BiometricPrompt.ERROR_CANCELED,
                -> CANCELLED

                BiometricPrompt.ERROR_HW_UNAVAILABLE,
                BiometricPrompt.ERROR_HW_NOT_PRESENT,
                BiometricPrompt.ERROR_NO_BIOMETRICS,
                BiometricPrompt.ERROR_LOCKOUT,
                BiometricPrompt.ERROR_LOCKOUT_PERMANENT,
                -> UNAVAILABLE

                else -> FAILED
            }

        /** What Dart is told once Rust replied [reply] to an answer of [status]. */
        fun outcomeAfter(
            status: BiometricStatus,
            reply: Int,
        ): BiometricOutcome =
            when (reply) {
                1 -> status.outcome
                -1 -> BiometricOutcome.INVALIDATED
                else -> BiometricOutcome.FAILED
            }

        /**
         * Whether the platform key and stored secret go after an answer of
         * [status] that ended as [outcome]: when biometrics changed, or when
         * an enrolment's secret was not kept (as iOS's `finish` and `enroll`
         * do). A failed unlock leaves them.
         */
        fun shouldForget(
            status: BiometricStatus,
            outcome: BiometricOutcome,
            enrolling: Boolean,
        ): Boolean =
            outcome == BiometricOutcome.INVALIDATED ||
                (enrolling && status == OK && outcome != BiometricOutcome.DONE)

        /** The kind to report for the biometric features the phone declares. */
        fun kindOf(
            fingerprint: Boolean,
            face: Boolean,
            iris: Boolean,
        ): BiometricKind =
            when {
                fingerprint && !face && !iris -> BiometricKind.FINGERPRINT
                face && !fingerprint && !iris -> BiometricKind.FACE
                iris && !fingerprint && !face -> BiometricKind.IRIS
                else -> BiometricKind.BIOMETRIC
            }
    }
}
