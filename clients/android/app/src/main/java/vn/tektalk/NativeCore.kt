package vn.tektalk

/** JNI entrypoint for the shared Rust Core. Debug builds stay usable before
 * native artifacts are produced, but release CI requires every ABI library. */
object NativeCore {
    val available: Boolean = runCatching { System.loadLibrary("tektalk_client_core"); true }.getOrDefault(false)
    @JvmStatic private external fun nativeSnowflake(nodeId: Long): Long
    @JvmStatic private external fun nativeMtprotoMessageId(clientToServer: Boolean): Long
    fun snowflake(nodeId: Long = 17): Long? = if (available) nativeSnowflake(nodeId).takeIf { it >= 0 } else null
    fun mtprotoMessageId(): Long? = if (available) nativeMtprotoMessageId(true).takeIf { it >= 0 } else null
}
