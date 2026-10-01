//! Cross-platform deterministic state for the TEKtalk mobile clients.
//!
//! Swift and Kotlin/JNI consume the stable C ABI declared in
//! `include/tektalk/ffi.h`; Rust types and ownership stay inside this crate.

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
}

#[no_mangle]
pub extern "C" fn tektalk_session_create(session_id: i64) -> *mut TektalkSession {
    Box::into_raw(Box::new(TektalkSession {
        state: SessionState::new(session_id),
    }))
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
}
