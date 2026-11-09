package com.kenchami.kenchamios.sync

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import java.io.File
import java.security.GeneralSecurityException
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/** Where the device credential is kept. It is a secret, so never the database or plain preferences. */
interface SecretStore {
    fun get(): String?

    fun set(value: String)

    fun clear()
}

private const val KEYSTORE = "AndroidKeyStore"
private const val KEY_ALIAS = "kenchamios-sync-credential"
private const val TRANSFORMATION = "AES/GCM/NoPadding"
private const val KEY_BITS = 256
private const val TAG_BITS = 128

/** AES-GCM with a key that never leaves the Android Keystore; only the encrypted credential is on disk. */
class KeystoreSecretStore(context: Context) : SecretStore {
    private val file = File(context.filesDir, "sync-credential.bin")

    override fun get(): String? {
        if (!file.exists()) return null
        return try {
            val bytes = file.readBytes()
            val ivLength = bytes[0].toInt()
            val cipher = Cipher.getInstance(TRANSFORMATION)
            cipher.init(Cipher.DECRYPT_MODE, key(), GCMParameterSpec(TAG_BITS, bytes, 1, ivLength))
            String(cipher.doFinal(bytes, 1 + ivLength, bytes.size - 1 - ivLength), Charsets.UTF_8)
        } catch (_: GeneralSecurityException) {
            // the key or the file is unusable (for example after a restore), the phone must pair again
            null
        }
    }

    override fun set(value: String) {
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.ENCRYPT_MODE, key())
        val encrypted = cipher.doFinal(value.toByteArray(Charsets.UTF_8))
        file.writeBytes(byteArrayOf(cipher.iv.size.toByte()) + cipher.iv + encrypted)
    }

    override fun clear() {
        file.delete()
    }

    private fun key(): SecretKey {
        val store = KeyStore.getInstance(KEYSTORE).apply { load(null) }
        (store.getKey(KEY_ALIAS, null) as? SecretKey)?.let { return it }
        val spec = KeyGenParameterSpec.Builder(KEY_ALIAS, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setKeySize(KEY_BITS)
            .build()
        return KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, KEYSTORE).apply { init(spec) }.generateKey()
    }
}
