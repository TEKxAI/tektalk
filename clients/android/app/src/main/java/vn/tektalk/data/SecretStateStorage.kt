package vn.tektalk.data

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.AtomicFile
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

interface SecretStateStorage {
    fun load(account:String):String?
    fun save(account:String,state:String)
}

/** Authenticated local snapshot, excluded from backup; AES key stays in Keystore. */
class AndroidSecretStateStorage(private val context:Context):SecretStateStorage {
    private fun key():SecretKey {
        val store=KeyStore.getInstance("AndroidKeyStore").apply{load(null)}
        val alias="tektalk.secret-chat.state.v1"
        (store.getKey(alias,null) as? SecretKey)?.let{return it}
        val generator=KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES,"AndroidKeyStore")
        generator.init(KeyGenParameterSpec.Builder(alias,KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM).setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE).build())
        return generator.generateKey()
    }
    private fun file(account:String):AtomicFile {
        require(account.matches(Regex("[a-fA-F0-9:-]+")))
        val directory=context.noBackupFilesDir.resolve("secret-chat").apply{mkdirs()}
        return AtomicFile(directory.resolve(account.replace(':','_')+".sealed"))
    }
    @Synchronized override fun load(account:String):String? {
        val file=file(account)
        if(!file.baseFile.exists())return null
        val bytes=file.readFully();require(bytes.size>28)
        val cipher=Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.DECRYPT_MODE,key(),GCMParameterSpec(128,bytes.copyOfRange(0,12)))
        cipher.updateAAD(account.toByteArray())
        val plaintext=cipher.doFinal(bytes.copyOfRange(12,bytes.size))
        return try{plaintext.toString(Charsets.UTF_8)}finally{plaintext.fill(0)}
    }
    @Synchronized override fun save(account:String,state:String) {
        val cipher=Cipher.getInstance("AES/GCM/NoPadding");cipher.init(Cipher.ENCRYPT_MODE,key());cipher.updateAAD(account.toByteArray())
        val plaintext=state.toByteArray()
        val encrypted=try{cipher.iv+cipher.doFinal(plaintext)}finally{plaintext.fill(0)}
        val file=file(account);val output=file.startWrite()
        try{output.write(encrypted);file.finishWrite(output)}catch(error:Throwable){file.failWrite(output);throw error}
    }
}
