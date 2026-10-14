use std::ffi::c_void;

use jni::{
    objects::{JByteArray, JClass, JString},
    sys::{jboolean, jlong},
    JNIEnv,
};
use crate::{id::SnowflakeGenerator, mtproto::{Direction, MessageIdGenerator}};
use crate::storage::AccountStore;

#[no_mangle]
pub extern "system" fn Java_vn_tektalk_NativeCore_nativeSnowflake(
    _env: *mut c_void,
    _class: *mut c_void,
    node_id: i64,
) -> i64 {
    u16::try_from(node_id)
        .ok()
        .and_then(|node| SnowflakeGenerator::new(node).ok())
        .and_then(|generator| generator.next_id().ok())
        .unwrap_or(-1)
}

#[no_mangle]
pub extern "system" fn Java_vn_tektalk_NativeCore_nativeMtprotoMessageId(
    _env: *mut c_void,
    _class: *mut c_void,
    client_to_server: u8,
) -> i64 {
    let direction = if client_to_server != 0 { Direction::ClientToServer } else { Direction::ServerToClient };
    MessageIdGenerator::new(direction).next_id().unwrap_or(-1)
}

#[no_mangle]
pub extern "system" fn Java_vn_tektalk_NativeCore_nativeDatabaseOpen(
    mut env: JNIEnv,
    _class: JClass,
    root: JString,
    key: JByteArray,
    max_open: jlong,
) -> jlong {
    let Ok(root) = env.get_string(&root).map(|value| value.to_string_lossy().into_owned()) else {
        return 0;
    };
    let Ok(key) = env.convert_byte_array(&key) else { return 0; };
    AccountStore::open(root, &key, usize::try_from(max_open).unwrap_or(8))
        .map(|store| Box::into_raw(Box::new(store)) as jlong)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "system" fn Java_vn_tektalk_NativeCore_nativeDatabaseClose(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    if handle != 0 {
        // SAFETY: Kotlin owns the handle and closes it exactly once.
        drop(unsafe { Box::from_raw(handle as *mut AccountStore) });
    }
}

#[no_mangle]
pub extern "system" fn Java_vn_tektalk_NativeCore_nativeDatabaseOpenConversation(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
    conversation_id: jlong,
) -> jboolean {
    if handle == 0 { return 0; }
    // SAFETY: Kotlin keeps the owning handle alive for this call.
    let store = unsafe { &*(handle as *const AccountStore) };
    jboolean::from(store.conversation(conversation_id).is_ok())
}
