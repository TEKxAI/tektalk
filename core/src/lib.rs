//! Cross-platform deterministic state for the TEKtalk mobile clients.
//!
//! Swift and Kotlin/JNI consume the stable C ABI declared in
//! `include/tektalk/ffi.h`; Rust types and ownership stay inside this crate.

pub mod id;
pub mod mtproto;
pub mod plugin;
pub mod transport;

#[cfg(target_os = "android")]
mod android_jni;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Direction {
    ClientToServer,
    ServerToClient,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TektalkAcceptResult {
    Accepted = 0,
    InvalidParity = 1,
    Replayed = 2,
}

#[derive(Debug)]
struct SessionState {
    _session_id: i64,
    content_sequence: i32,
    last_client_message_id: Option<i64>,
    last_server_message_id: Option<i64>,
}

impl SessionState {
    fn new(session_id: i64) -> Self {
        Self {
            _session_id: session_id,
            content_sequence: 0,
            last_client_message_id: None,
            last_server_message_id: None,
        }
    }

    fn next_content_sequence(&mut self) -> Option<i32> {
        let sequence = self.content_sequence.checked_mul(2)?.checked_add(1)?;
        self.content_sequence = self.content_sequence.checked_add(1)?;
        Some(sequence)
    }

    fn accept_message_id(
        &mut self,
        message_id: i64,
        direction: Direction,
    ) -> TektalkAcceptResult {
        let modulo = message_id.rem_euclid(4);
        let valid = match direction {
            Direction::ClientToServer => modulo == 0,
            Direction::ServerToClient => modulo == 1 || modulo == 3,
        };
        if !valid {
            return TektalkAcceptResult::InvalidParity;
        }

        let last = match direction {
            Direction::ClientToServer => &mut self.last_client_message_id,
            Direction::ServerToClient => &mut self.last_server_message_id,
        };
        if last.is_some_and(|previous| message_id <= previous) {
            return TektalkAcceptResult::Replayed;
        }
        *last = Some(message_id);
        TektalkAcceptResult::Accepted
    }
}

/// Opaque owner for a client session crossing the C ABI.
#[repr(C)]
pub struct TektalkSession {
    state: SessionState,
    client_message_ids: mtproto::MessageIdGenerator,
    server_message_ids: mtproto::MessageIdGenerator,
}

#[repr(C)]
pub struct TektalkSnowflake {
    generator: id::SnowflakeGenerator,
}

#[no_mangle]
pub extern "C" fn tektalk_session_create(session_id: i64) -> *mut TektalkSession {
    Box::into_raw(Box::new(TektalkSession {
        state: SessionState::new(session_id),
        client_message_ids: mtproto::MessageIdGenerator::new(mtproto::Direction::ClientToServer),
        server_message_ids: mtproto::MessageIdGenerator::new(mtproto::Direction::ServerToClient),
    }))
}

#[no_mangle]
pub extern "C" fn tektalk_snowflake_create(node_id: u16) -> *mut TektalkSnowflake {
    let Ok(generator) = id::SnowflakeGenerator::new(node_id) else {
        return std::ptr::null_mut();
    };
    Box::into_raw(Box::new(TektalkSnowflake { generator }))
}

/// # Safety
///
/// A non-null pointer must be a uniquely owned live handle returned by
/// [`tektalk_snowflake_create`].
#[no_mangle]
pub unsafe extern "C" fn tektalk_snowflake_destroy(generator: *mut TektalkSnowflake) {
    if !generator.is_null() {
        // SAFETY: Required by the public function contract.
        drop(unsafe { Box::from_raw(generator) });
    }
}

/// # Safety
///
/// A non-null pointer must be a live handle returned by
/// [`tektalk_snowflake_create`]. Access must be serialized by the caller.
#[no_mangle]
pub unsafe extern "C" fn tektalk_snowflake_next(generator: *mut TektalkSnowflake) -> i64 {
    // SAFETY: Required by the public function contract. Null is handled.
    unsafe { generator.as_ref() }
        .and_then(|generator| generator.generator.next_id().ok())
        .unwrap_or(-1)
}

/// Releases a session allocated by [`tektalk_session_create`].
///
/// A null pointer is accepted. Any non-null pointer must have been returned by
/// `tektalk_session_create` and must not be released more than once.
///
/// # Safety
///
/// A non-null `session` must be a live handle returned by
/// [`tektalk_session_create`] and uniquely owned by the caller.
#[no_mangle]
pub unsafe extern "C" fn tektalk_session_destroy(session: *mut TektalkSession) {
    if !session.is_null() {
        // SAFETY: The caller contract requires unique ownership of a pointer
        // created by Box::into_raw in tektalk_session_create.
        drop(unsafe { Box::from_raw(session) });
    }
}

/// Returns the next odd content-related sequence number, or `-1` for a null or
/// exhausted session.
///
/// # Safety
///
/// A non-null `session` must be a live handle returned by
/// [`tektalk_session_create`]. Access to a handle must be serialized.
#[no_mangle]
pub unsafe extern "C" fn tektalk_session_next_content_sequence(
    session: *mut TektalkSession,
) -> i32 {
    // SAFETY: as_mut only creates a reference for a non-null pointer. The C ABI
    // requires the caller to serialize access to each opaque session handle.
    unsafe { session.as_mut() }
        .and_then(|session| session.state.next_content_sequence())
        .unwrap_or(-1)
}

/// Validates message-ID direction parity and monotonically increasing order.
///
/// # Safety
///
/// A non-null `session` must be a live handle returned by
/// [`tektalk_session_create`]. Access to a handle must be serialized.
#[no_mangle]
pub unsafe extern "C" fn tektalk_session_accept_message_id(
    session: *mut TektalkSession,
    message_id: i64,
    client_to_server: i32,
) -> TektalkAcceptResult {
    // SAFETY: See tektalk_session_next_content_sequence. A null handle maps to
    // the legacy conservative result and is never dereferenced.
    let Some(session) = (unsafe { session.as_mut() }) else {
        return TektalkAcceptResult::Replayed;
    };
    let direction = if client_to_server != 0 {
        Direction::ClientToServer
    } else {
        Direction::ServerToClient
    };
    session.state.accept_message_id(message_id, direction)
}

/// Generates an MTProto 2.0-compliant wire message ID.
///
/// # Safety
///
/// A non-null `session` must be a live handle returned by
/// [`tektalk_session_create`]. Access to a handle must be serialized.
#[no_mangle]
pub unsafe extern "C" fn tektalk_session_next_mtproto_message_id(
    session: *mut TektalkSession,
    client_to_server: i32,
) -> i64 {
    // SAFETY: Required by the public function contract. Null is handled.
    let Some(session) = (unsafe { session.as_ref() }) else { return -1; };
    let generator = if client_to_server != 0 {
        &session.client_message_ids
    } else {
        &session.server_message_ids
    };
    generator.next_id().unwrap_or(-1)
}

/// Verifies SHA-256 and Ed25519 before a downloaded plugin may be loaded.
///
/// # Safety
/// `bytes` must reference `length` readable bytes. The remaining pointers must
/// reference 32, 64 and 32 readable bytes respectively.
#[no_mangle]
pub unsafe extern "C" fn tektalk_plugin_verify_artifact(
    bytes: *const u8,
    length: usize,
    expected_sha256: *const u8,
    signature: *const u8,
    public_key: *const u8,
) -> i32 {
    if bytes.is_null() || expected_sha256.is_null() || signature.is_null() || public_key.is_null() {
        return 0;
    }
    // SAFETY: buffer sizes are required by the public C ABI contract.
    let artifact = unsafe { std::slice::from_raw_parts(bytes, length) };
    let digest: &[u8; 32] = unsafe { std::slice::from_raw_parts(expected_sha256, 32) }.try_into().expect("fixed length");
    let signature: &[u8; 64] = unsafe { std::slice::from_raw_parts(signature, 64) }.try_into().expect("fixed length");
    let public_key: &[u8; 32] = unsafe { std::slice::from_raw_parts(public_key, 32) }.try_into().expect("fixed length");
    if plugin::verify_artifact(artifact, digest, signature, public_key) { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn preserves_session_id_and_generates_odd_content_sequences() {
        let mut session = SessionState::new(42);
        assert_eq!(session._session_id, 42);
        assert_eq!(session.next_content_sequence(), Some(1));
        assert_eq!(session.next_content_sequence(), Some(3));
        assert_eq!(session.next_content_sequence(), Some(5));
    }

    #[test]
    fn rejects_sequence_overflow_without_panicking() {
        let mut session = SessionState::new(1);
        session.content_sequence = i32::MAX / 2 + 1;
        assert_eq!(session.next_content_sequence(), None);
    }

    #[test]
    fn enforces_direction_parity_and_replay_order() {
        let mut session = SessionState::new(42);
        assert_eq!(
            session.accept_message_id(0x1_0000_0000, Direction::ClientToServer),
            TektalkAcceptResult::Accepted
        );
        assert_eq!(
            session.accept_message_id(0x1_0000_0000, Direction::ClientToServer),
            TektalkAcceptResult::Replayed
        );
        assert_eq!(
            session.accept_message_id(0x1_0000_0002, Direction::ClientToServer),
            TektalkAcceptResult::InvalidParity
        );
        assert_eq!(
            session.accept_message_id(0x1_0000_0001, Direction::ServerToClient),
            TektalkAcceptResult::Accepted
        );
        assert_eq!(
            session.accept_message_id(0x1_0000_0005, Direction::ServerToClient),
            TektalkAcceptResult::Accepted
        );
        assert_eq!(
            session.accept_message_id(0x1_0000_0003, Direction::ServerToClient),
            TektalkAcceptResult::Replayed
        );
    }

    #[test]
    fn tracks_directions_independently_and_normalizes_negative_modulo() {
        let mut session = SessionState::new(7);
        assert_eq!(
            session.accept_message_id(4, Direction::ClientToServer),
            TektalkAcceptResult::Accepted
        );
        assert_eq!(
            session.accept_message_id(1, Direction::ServerToClient),
            TektalkAcceptResult::Accepted
        );
        assert_eq!(
            session.accept_message_id(-4, Direction::ClientToServer),
            TektalkAcceptResult::Replayed
        );

        let mut negative = SessionState::new(8);
        assert_eq!(
            negative.accept_message_id(-4, Direction::ClientToServer),
            TektalkAcceptResult::Accepted
        );
        assert_eq!(
            negative.accept_message_id(-3, Direction::ServerToClient),
            TektalkAcceptResult::Accepted
        );
    }

    #[test]
    fn ffi_round_trip_preserves_legacy_contract() {
        let session = tektalk_session_create(99);
        assert!(!session.is_null());
        // SAFETY: session is live and access is serialized in this test.
        unsafe {
            assert_eq!(tektalk_session_next_content_sequence(session), 1);
            assert_eq!(
                tektalk_session_accept_message_id(session, 4, 1),
                TektalkAcceptResult::Accepted
            );
            assert_eq!(
                tektalk_session_accept_message_id(session, 4, 1),
                TektalkAcceptResult::Replayed
            );
        }
        // SAFETY: session was created above and is released exactly once.
        unsafe { tektalk_session_destroy(session) };
    }

    #[test]
    fn ffi_null_handle_is_conservative() {
        // SAFETY: Null is explicitly accepted by the ABI.
        unsafe {
            assert_eq!(tektalk_session_next_content_sequence(ptr::null_mut()), -1);
            assert_eq!(
                tektalk_session_accept_message_id(ptr::null_mut(), 4, 1),
                TektalkAcceptResult::Replayed
            );
            tektalk_session_destroy(ptr::null_mut());
        }
    }

    #[test]
    fn ffi_discriminants_match_c_header() {
        assert_eq!(TektalkAcceptResult::Accepted as i32, 0);
        assert_eq!(TektalkAcceptResult::InvalidParity as i32, 1);
        assert_eq!(TektalkAcceptResult::Replayed as i32, 2);
    }

    #[test]
    fn ffi_rejects_null_plugin_artifact() {
        assert_eq!(unsafe { tektalk_plugin_verify_artifact(ptr::null(), 0, ptr::null(), ptr::null(), ptr::null()) }, 0);
    }

    #[test]
    fn ffi_generators_enforce_their_distinct_layouts() {
        let snowflake = tektalk_snowflake_create(17);
        let session = tektalk_session_create(1);
        unsafe {
            let persisted = tektalk_snowflake_next(snowflake);
            assert_eq!(id::SnowflakeGenerator::decode(persisted).node_id, 17);
            let client_wire = tektalk_session_next_mtproto_message_id(session, 1);
            let server_wire = tektalk_session_next_mtproto_message_id(session, 0);
            assert_eq!(client_wire.rem_euclid(4), 0);
            assert_eq!(server_wire.rem_euclid(4), 1);
            tektalk_snowflake_destroy(snowflake);
            tektalk_session_destroy(session);
        }
    }
}
