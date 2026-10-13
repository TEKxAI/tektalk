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
    fun snowflake(nodeId: Long = 17): Long? = if (available) nativeSnowflake(nodeId).takeIf { it >= 0 } else null
    fun mtprotoMessageId(): Long? = if (available) nativeMtprotoMessageId(true).takeIf { it >= 0 } else null
    fun openDatabase(root: String, key: ByteArray, maxOpen: Long = 8): Long? =
        if (available && key.size == 32) nativeDatabaseOpen(root, key, maxOpen).takeIf { it != 0L } else null
    fun closeDatabase(handle: Long) = nativeDatabaseClose(handle)
    fun openConversation(handle: Long, conversationId: Long): Boolean =
        available && conversationId > 0 && nativeDatabaseOpenConversation(handle, conversationId)
}
