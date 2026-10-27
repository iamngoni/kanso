package app.kanso.kanso_lock

import androidx.biometric.BiometricPrompt
import org.junit.Test
import org.junit.Assert.assertEquals

class BiometricStatusTest {
    @Test
    fun cancelCodesAreCancelled() {
        for (code in listOf(
            BiometricPrompt.ERROR_USER_CANCELED,
            BiometricPrompt.ERROR_NEGATIVE_BUTTON,
            BiometricPrompt.ERROR_CANCELED,
        )) {
            assertEquals(BiometricStatus.CANCELLED, BiometricStatus.ofPromptError(code))
        }
    }

    @Test
    fun hardwareAndLockoutCodesAreUnavailable() {
        for (code in listOf(
            BiometricPrompt.ERROR_HW_UNAVAILABLE,
            BiometricPrompt.ERROR_NO_BIOMETRICS,
            BiometricPrompt.ERROR_LOCKOUT,
            BiometricPrompt.ERROR_LOCKOUT_PERMANENT,
        )) {
            assertEquals(BiometricStatus.UNAVAILABLE, BiometricStatus.ofPromptError(code))
        }
    }

    @Test
    fun otherCodesFail() {
        assertEquals(BiometricStatus.FAILED, BiometricStatus.ofPromptError(BiometricPrompt.ERROR_TIMEOUT))
        assertEquals(BiometricStatus.FAILED, BiometricStatus.ofPromptError(-1))
    }

    @Test
    fun codesAreTheContracts() {
        assertEquals(listOf(0, 1, 2, 3, 4), BiometricStatus.values().map { it.code })
    }

    @Test
    fun rustsReplyDecidesTheOutcome() {
        assertEquals(BiometricOutcome.DONE, BiometricStatus.outcomeAfter(BiometricStatus.OK, 1))
        assertEquals(BiometricOutcome.CANCELLED, BiometricStatus.outcomeAfter(BiometricStatus.CANCELLED, 1))
        assertEquals(BiometricOutcome.INVALIDATED, BiometricStatus.outcomeAfter(BiometricStatus.OK, -1))
        assertEquals(BiometricOutcome.FAILED, BiometricStatus.outcomeAfter(BiometricStatus.OK, 0))
        assertEquals(BiometricOutcome.FAILED, BiometricStatus.outcomeAfter(BiometricStatus.OK, -5))
    }

    @Test
    fun theKeyIsForgottenOnlyWhenInvalidatedOrAnEnrolmentWasNotKept() {
        val forget = BiometricStatus::shouldForget
        // Invalidated, in either flow.
        assertEquals(true, forget(BiometricStatus.INVALIDATED, BiometricOutcome.INVALIDATED, false))
        assertEquals(true, forget(BiometricStatus.OK, BiometricOutcome.INVALIDATED, false))
        // An unlock Rust could not apply (no ticket, a fault) keeps the key.
        assertEquals(false, forget(BiometricStatus.OK, BiometricOutcome.FAILED, false))
        assertEquals(false, forget(BiometricStatus.OK, BiometricOutcome.DONE, false))
        // An enrolment Rust did not keep forgets it; one it kept does not.
        assertEquals(true, forget(BiometricStatus.OK, BiometricOutcome.FAILED, true))
        assertEquals(false, forget(BiometricStatus.OK, BiometricOutcome.DONE, true))
        // Cancelled and the like never do.
        assertEquals(false, forget(BiometricStatus.CANCELLED, BiometricOutcome.CANCELLED, true))
    }

    @Test
    fun oneFeatureNamesTheKindManyDoNot() {
        assertEquals(BiometricKind.FINGERPRINT, BiometricStatus.kindOf(true, false, false))
        assertEquals(BiometricKind.FACE, BiometricStatus.kindOf(false, true, false))
        assertEquals(BiometricKind.IRIS, BiometricStatus.kindOf(false, false, true))
        assertEquals(BiometricKind.BIOMETRIC, BiometricStatus.kindOf(true, true, false))
        assertEquals(BiometricKind.BIOMETRIC, BiometricStatus.kindOf(false, false, false))
    }
}
