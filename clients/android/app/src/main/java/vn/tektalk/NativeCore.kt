package vn.tektalk

/** JNI entrypoint for the shared Rust Core. Debug builds stay usable before
 * native artifacts are produced, but release CI requires every ABI library. */
object NativeCore {
    val available: Boolean = runCatching { System.loadLibrary("tektalk_client_core"); true }.getOrDefault(false)
    @JvmStatic private external fun nativeSnowflake(nodeId: Long): Long
    @JvmStatic private external fun nativeMtprotoMessageId(clientToServer: Boolean): Long
    @JvmStatic private external fun nativeDatabaseOpen(root: String, key: ByteArray, maxOpen: Long): Long
    @JvmStatic private external fun nativeDatabaseClose(handle: Long)
    @JvmStatic private external fun nativeDatabaseOpenConversation(handle: Long, conversationId: Long): Boolean
    @JvmStatic private external fun nativeSecretCreate(userId: String, deviceId: String): Long
    @JvmStatic private external fun nativeSecretDestroy(handle: Long)
    @JvmStatic private external fun nativeSecretBundle(handle: Long): String?
    @JvmStatic private external fun nativeSecretBegin(handle: Long, conversationId: String, kind: String, epoch: Long, bundlesJson: String): String?
    @JvmStatic private external fun nativeSecretAccept(handle: Long, conversationId: String, senderDeviceId: String, kind: String, epoch: Long, wrappedJson: String): Boolean
    @JvmStatic private external fun nativeSecretEncrypt(handle: Long, conversationId: String, epoch: Long, plaintext: String): String?
    @JvmStatic private external fun nativeSecretDecrypt(handle: Long, messageJson: String): String?
    fun snowflake(nodeId: Long = 17): Long? = if (available) nativeSnowflake(nodeId).takeIf { it >= 0 } else null
    fun mtprotoMessageId(): Long? = if (available) nativeMtprotoMessageId(true).takeIf { it >= 0 } else null
    fun openDatabase(root: String, key: ByteArray, maxOpen: Long = 8): Long? =
        if (available && key.size == 32) nativeDatabaseOpen(root, key, maxOpen).takeIf { it != 0L } else null
    fun closeDatabase(handle: Long) = nativeDatabaseClose(handle)
    fun openConversation(handle: Long, conversationId: Long): Boolean =
        available && conversationId > 0 && nativeDatabaseOpenConversation(handle, conversationId)

    class SecretDevice internal constructor(private var handle:Long):AutoCloseable {
        val publicBundleJson:String get()=checkNotNull(nativeSecretBundle(handle)){"Secret Chat core rejected device state"}
        fun beginEpoch(conversationId:String,kind:String,epoch:Long,bundlesJson:String):String = checkNotNull(nativeSecretBegin(handle,conversationId,kind,epoch,bundlesJson)){"Secret Chat key distribution failed"}
        fun acceptKey(conversationId:String,senderDeviceId:String,kind:String,epoch:Long,wrappedJson:String):Boolean = nativeSecretAccept(handle,conversationId,senderDeviceId,kind,epoch,wrappedJson)
        fun encrypt(conversationId:String,epoch:Long,plaintext:String):String = checkNotNull(nativeSecretEncrypt(handle,conversationId,epoch,plaintext)){"Secret Chat is not ready for this epoch"}
        fun decrypt(messageJson:String):String = checkNotNull(nativeSecretDecrypt(handle,messageJson)){"Secret Chat authentication or replay check failed"}
        override fun close(){if(handle!=0L){nativeSecretDestroy(handle);handle=0}}
    }
    fun secretDevice(userId:String,deviceId:String):SecretDevice? = if(available) nativeSecretCreate(userId,deviceId).takeIf{it!=0L}?.let(::SecretDevice) else null
}
