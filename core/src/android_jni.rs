use std::ffi::{c_void, CStr, CString};

use jni::{
    objects::{JByteArray, JClass, JString},
    sys::{jboolean, jlong, jstring},
    JNIEnv,
};
use crate::{id::SnowflakeGenerator, mtproto::{Direction, MessageIdGenerator}};
use crate::storage::AccountStore;
use crate::TektalkSecretDevice;

fn java_string(mut env: JNIEnv, value: *mut std::ffi::c_char) -> jstring {
    if value.is_null() { return std::ptr::null_mut(); }
    // SAFETY: All Secret Chat string results are allocated by CString::into_raw.
    let text = unsafe { CStr::from_ptr(value) }.to_string_lossy().into_owned();
    // SAFETY: Release the Rust-owned result exactly once after copying it.
    unsafe { crate::tektalk_string_destroy(value) };
    env.new_string(text).map(|s| s.into_raw()).unwrap_or(std::ptr::null_mut())
}

fn rust_string(env: &mut JNIEnv, value: JString) -> Option<CString> {
    env.get_string(&value).ok().and_then(|s| CString::new(s.to_bytes()).ok())
}

#[no_mangle]
pub extern "system" fn Java_vn_tektalk_NativeCore_nativeSecretSnapshot(env: JNIEnv, _class: JClass, handle: jlong) -> jstring {
    // SAFETY: Kotlin serializes access to the owning device handle.
    java_string(env, unsafe { crate::tektalk_secret_snapshot_json(handle as *const TektalkSecretDevice) })
}

#[no_mangle]
pub extern "system" fn Java_vn_tektalk_NativeCore_nativeSecretRestore(mut env: JNIEnv, _class: JClass, snapshot: JString) -> jlong {
    let Some(snapshot) = rust_string(&mut env, snapshot) else { return 0; };
    // SAFETY: The CString remains live throughout the restore call.
    unsafe { crate::tektalk_secret_restore_json(snapshot.as_ptr()) as jlong }
}

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

#[no_mangle]
pub extern "system" fn Java_vn_tektalk_NativeCore_nativeSecretCreate(mut env: JNIEnv, _class: JClass, user_id: JString, device_id: JString) -> jlong {
    let Some(user_id) = rust_string(&mut env, user_id) else { return 0; };
    let Some(device_id) = rust_string(&mut env, device_id) else { return 0; };
    // SAFETY: CString values are live for the duration of the call.
    unsafe { crate::tektalk_secret_device_create(user_id.as_ptr(), device_id.as_ptr()) as jlong }
}

#[no_mangle]
pub extern "system" fn Java_vn_tektalk_NativeCore_nativeSecretDestroy(_env: JNIEnv, _class: JClass, handle: jlong) {
    // SAFETY: Kotlin owns this handle and releases it exactly once.
    unsafe { crate::tektalk_secret_device_destroy(handle as *mut TektalkSecretDevice) }
}

#[no_mangle]
pub extern "system" fn Java_vn_tektalk_NativeCore_nativeSecretBundle(env: JNIEnv, _class: JClass, handle: jlong) -> jstring {
    // SAFETY: Kotlin keeps the owning handle alive during this call.
    java_string(env, unsafe { crate::tektalk_secret_public_bundle_json(handle as *const TektalkSecretDevice) })
}

#[no_mangle]
pub extern "system" fn Java_vn_tektalk_NativeCore_nativeSecretBegin(mut env: JNIEnv, _class: JClass, handle: jlong, conversation: JString, kind: JString, epoch: jlong, bundles: JString) -> jstring {
    let Some(conversation) = rust_string(&mut env, conversation) else { return std::ptr::null_mut(); };
    let Some(kind) = rust_string(&mut env, kind) else { return std::ptr::null_mut(); };
    let Some(bundles) = rust_string(&mut env, bundles) else { return std::ptr::null_mut(); };
    java_string(env, unsafe { crate::tektalk_secret_begin_epoch_json(handle as *mut TektalkSecretDevice, conversation.as_ptr(), kind.as_ptr(), epoch as u64, bundles.as_ptr()) })
}

#[no_mangle]
pub extern "system" fn Java_vn_tektalk_NativeCore_nativeSecretAccept(mut env: JNIEnv, _class: JClass, handle: jlong, conversation: JString, sender: JString, kind: JString, epoch: jlong, wrapped: JString) -> jboolean {
    let Some(conversation) = rust_string(&mut env, conversation) else { return 0; };
    let Some(sender) = rust_string(&mut env, sender) else { return 0; };
    let Some(kind) = rust_string(&mut env, kind) else { return 0; };
    let Some(wrapped) = rust_string(&mut env, wrapped) else { return 0; };
    jboolean::from(unsafe { crate::tektalk_secret_accept_key_json(handle as *mut TektalkSecretDevice, conversation.as_ptr(), sender.as_ptr(), kind.as_ptr(), epoch as u64, wrapped.as_ptr()) } == 1)
}

#[no_mangle]
pub extern "system" fn Java_vn_tektalk_NativeCore_nativeSecretEncrypt(mut env: JNIEnv, _class: JClass, handle: jlong, conversation: JString, epoch: jlong, plaintext: JString) -> jstring {
    let Some(conversation) = rust_string(&mut env, conversation) else { return std::ptr::null_mut(); };
    let Some(plaintext) = rust_string(&mut env, plaintext) else { return std::ptr::null_mut(); };
    java_string(env, unsafe { crate::tektalk_secret_encrypt_json(handle as *mut TektalkSecretDevice, conversation.as_ptr(), epoch as u64, plaintext.as_ptr()) })
}

#[no_mangle]
pub extern "system" fn Java_vn_tektalk_NativeCore_nativeSecretDecrypt(mut env: JNIEnv, _class: JClass, handle: jlong, message: JString) -> jstring {
    let Some(message) = rust_string(&mut env, message) else { return std::ptr::null_mut(); };
    java_string(env, unsafe { crate::tektalk_secret_decrypt_json(handle as *mut TektalkSecretDevice, message.as_ptr()) })
}
