use std::ffi::c_void;

use crate::{id::SnowflakeGenerator, mtproto::{Direction, MessageIdGenerator}};

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
