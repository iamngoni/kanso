package app.kanso.kanso_lock

import android.content.Context
import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyPermanentlyInvalidatedException
import android.security.keystore.KeyProperties
import android.security.keystore.StrongBoxUnavailableException
import android.util.Base64
import java.security.KeyStore
import java.security.UnrecoverableKeyException
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/**
 * The biometric secret's wrapping key: AES-256-GCM in the Android Keystore,
 * usable only after a strong biometric check and invalidated when biometrics
 * are enrolled or removed. The secret is stored encrypted by it (ciphertext
 * and IV in the plugin's private preferences).
 */
class BiometricKeys(
    context: Context,
) {
    private val preferences =
        context.applicationContext.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE)

    /** A key was lost or no longer opens: the stored secret is of no use. */
    class Gone : Exception()

    /** A new key for [libraryId] and a cipher to encrypt with it. */
    fun newEncryptCipher(libraryId: String): Cipher {
        forget(libraryId)
        val key = generate(alias(libraryId))
        return Cipher.getInstance(TRANSFORMATION).apply { init(Cipher.ENCRYPT_MODE, key) }
    }

    /** Stores [secret] sealed by [cipher] (the prompt's, after the check). */
    fun store(
        libraryId: String,
        cipher: Cipher,
        secret: ByteArray,
    ) {
        val sealed = cipher.doFinal(secret)
        // Written before the enrolment is told to Rust, and off the main
        // thread already, so it is waited for.
        val written =
            preferences
                .edit()
                .putString(prefKey(libraryId, "ct"), Base64.encodeToString(sealed, Base64.NO_WRAP))
                .putString(prefKey(libraryId, "iv"), Base64.encodeToString(cipher.iv, Base64.NO_WRAP))
                .commit()
        check(written) { "the sealed secret was not written" }
    }

    /** A cipher to decrypt [libraryId]'s secret; [Gone] when it cannot be. */
    fun decryptCipher(libraryId: String): Cipher {
        val iv = preferences.getString(prefKey(libraryId, "iv"), null)
        // A key the Keystore can no longer recover is gone, like one it lost.
        val key =
            try {
                keyStore().getKey(alias(libraryId), null) as? SecretKey
            } catch (error: UnrecoverableKeyException) {
                null
            }
        if (iv == null || key == null || sealed(libraryId) == null) {
            forget(libraryId)
            throw Gone()
        }
        try {
            return Cipher.getInstance(TRANSFORMATION).apply {
                init(Cipher.DECRYPT_MODE, key, GCMParameterSpec(TAG_BITS, Base64.decode(iv, Base64.NO_WRAP)))
            }
        } catch (error: KeyPermanentlyInvalidatedException) {
            forget(libraryId)
            throw Gone()
        }
    }

    /** The sealed secret of [libraryId], null when there is none. */
    fun sealed(libraryId: String): ByteArray? =
        preferences.getString(prefKey(libraryId, "ct"), null)?.let { Base64.decode(it, Base64.NO_WRAP) }

    /** Deletes the key and the stored secret of [libraryId]. */
    fun forget(libraryId: String) {
        try {
            keyStore().deleteEntry(alias(libraryId))
        } catch (error: Exception) {
            // Nothing to delete, or the Keystore is busy: the preferences go
            // anyway, and the next enrolment replaces the key.
        }
        // Waited for, as a store is: a forgotten secret must not come back
        // if the process dies before an asynchronous write lands.
        preferences
            .edit()
            .remove(prefKey(libraryId, "ct"))
            .remove(prefKey(libraryId, "iv"))
            .commit()
    }

    private fun generate(alias: String): SecretKey {
        try {
            return generator(alias, strongBox = true).generateKey()
        } catch (error: StrongBoxUnavailableException) {
            return generator(alias, strongBox = false).generateKey()
        }
    }

    private fun generator(
        alias: String,
        strongBox: Boolean,
    ): KeyGenerator {
        val spec =
            KeyGenParameterSpec
                .Builder(alias, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setKeySize(256)
                .setUserAuthenticationRequired(true)
                .setInvalidatedByBiometricEnrollment(true)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            spec.setUserAuthenticationParameters(0, KeyProperties.AUTH_BIOMETRIC_STRONG)
        } else {
            @Suppress("DEPRECATION")
            spec.setUserAuthenticationValidityDurationSeconds(-1)
        }
        if (strongBox) {
            spec.setIsStrongBoxBacked(true)
        }
        return KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, KEYSTORE).apply {
            init(spec.build())
        }
    }

    private fun keyStore(): KeyStore = KeyStore.getInstance(KEYSTORE).apply { load(null) }

    companion object {
        const val PREFERENCES = "kanso_lock"
        private const val KEYSTORE = "AndroidKeyStore"
        private const val TRANSFORMATION = "AES/GCM/NoPadding"
        private const val TAG_BITS = 128

        /** The Keystore alias of [libraryId]'s key. */
        fun alias(libraryId: String) = "kanso.biometric.$libraryId"

        private fun prefKey(
            libraryId: String,
            part: String,
        ) = "$libraryId.$part"
    }
}
