package vn.tektalk.data

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import java.security.KeyStore
import java.security.SecureRandom
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec
import vn.tektalk.NativeCore

/** SQLCipher account store whose random 256-bit key is wrapped by Android Keystore. */
class EncryptedDatabase private constructor(private var nativeHandle: Long) : AutoCloseable {
    fun openConversation(conversationId: Long): Boolean =
        nativeHandle != 0L && NativeCore.openConversation(nativeHandle, conversationId)

    override fun close() {
        if (nativeHandle != 0L) {
            NativeCore.closeDatabase(nativeHandle)
            nativeHandle = 0
        }
    }

    companion object {
        fun open(context: Context, accountId: Long): EncryptedDatabase {
            require(accountId > 0)
            val root = context.noBackupFilesDir.resolve("accounts/$accountId").apply { mkdirs() }
            val key = AndroidDatabaseKeyProvider(context).getOrCreate(accountId)
            try {
                val handle = requireNotNull(NativeCore.openDatabase(root.absolutePath, key)) {
                    "SQLCipher database could not be opened"
                }
                return EncryptedDatabase(handle)
            } finally {
                key.fill(0)
            }
        }
    }
}

private class AndroidDatabaseKeyProvider(context: Context) {
    private val preferences = context.getSharedPreferences("tektalk.database.keys", Context.MODE_PRIVATE)
    private val keyStore = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }

    @Synchronized
    fun getOrCreate(accountId: Long): ByteArray {
        val alias = "tektalk.database.wrap.$accountId"
        val wrapped = preferences.getString(alias, null)
        if (wrapped != null) return unwrap(alias, Base64.decode(wrapped, Base64.NO_WRAP))
        val databaseKey = ByteArray(32).also(SecureRandom()::nextBytes)
        val encoded = wrap(alias, databaseKey)
        check(preferences.edit().putString(alias, Base64.encodeToString(encoded, Base64.NO_WRAP)).commit())
        return databaseKey
    }

    private fun wrappingKey(alias: String): SecretKey {
        (keyStore.getKey(alias, null) as? SecretKey)?.let { return it }
        val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore")
        generator.init(KeyGenParameterSpec.Builder(alias, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setRandomizedEncryptionRequired(true)
            .build())
        return generator.generateKey()
    }

    private fun wrap(alias: String, plaintext: ByteArray): ByteArray {
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, wrappingKey(alias))
        cipher.updateAAD(alias.toByteArray(Charsets.UTF_8))
        return cipher.iv + cipher.doFinal(plaintext)
    }

    private fun unwrap(alias: String, encoded: ByteArray): ByteArray {
        require(encoded.size > 12)
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.DECRYPT_MODE, wrappingKey(alias), GCMParameterSpec(128, encoded.copyOfRange(0, 12)))
        cipher.updateAAD(alias.toByteArray(Charsets.UTF_8))
        return cipher.doFinal(encoded.copyOfRange(12, encoded.size))
    }
}
