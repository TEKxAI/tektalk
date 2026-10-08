use aes::{cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt, KeyInit}, Aes256};
use rand::{rngs::OsRng, RngCore};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::{sync::Mutex, time::{SystemTime, UNIX_EPOCH}};
use subtle::ConstantTimeEq;

pub const AUTH_KEY_LEN: usize = 256;
pub const EXTERNAL_HEADER_LEN: usize = 24;
pub const INTERNAL_HEADER_LEN: usize = 32;
const MIN_PADDING: usize = 12;
const MAX_PADDING: usize = 1024;

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum Error {
    #[error("{0}")]
    Invalid(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub server_salt: i64,
    pub session_id: i64,
    pub message_id: i64,
    pub sequence_no: i32,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction { ClientToServer, ServerToClient }

impl Direction {
    fn x(self) -> usize { match self { Self::ClientToServer => 0, Self::ServerToClient => 8 } }
    fn parity(self) -> u64 { match self { Self::ClientToServer => 0, Self::ServerToClient => 1 } }
}

/// Monotonic time-derived MTProto message IDs. This is deliberately separate
/// from persisted Snowflake IDs because MTProto defines direction parity.
#[derive(Debug)]
pub struct MessageIdGenerator {
    direction: Direction,
    last: Mutex<u64>,
}

impl MessageIdGenerator {
    pub fn new(direction: Direction) -> Self { Self { direction, last: Mutex::new(0) } }

    pub fn next_id(&self) -> Result<i64> {
        let duration = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| Error::Invalid("clock before Unix epoch"))?;
        let fraction = ((u64::from(duration.subsec_nanos()) << 32) / 1_000_000_000) & !3;
        let candidate = (duration.as_secs() << 32) | fraction | self.direction.parity();
        let mut last = self.last.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let next = if candidate > *last { candidate } else { last.checked_add(4).ok_or(Error::Invalid("message id exhausted"))? };
        *last = next;
        Ok(next as i64)
    }
}

pub fn auth_key_id(auth_key: &[u8; AUTH_KEY_LEN]) -> [u8; 8] {
    Sha1::digest(auth_key)[12..20].try_into().expect("fixed digest range")
}

pub fn encrypt(auth_key: &[u8; AUTH_KEY_LEN], message: &Message, direction: Direction) -> Result<Vec<u8>> {
    validate_message_id(message.message_id, direction)?;
    if message.body.len() > i32::MAX as usize { return Err(Error::Invalid("message body too large")); }
    let mut plaintext = Vec::with_capacity(INTERNAL_HEADER_LEN + message.body.len() + 32);
    plaintext.extend_from_slice(&message.server_salt.to_le_bytes());
    plaintext.extend_from_slice(&message.session_id.to_le_bytes());
    plaintext.extend_from_slice(&message.message_id.to_le_bytes());
    plaintext.extend_from_slice(&message.sequence_no.to_le_bytes());
    plaintext.extend_from_slice(&(message.body.len() as i32).to_le_bytes());
    plaintext.extend_from_slice(&message.body);
    let padding_len = padding_length(plaintext.len());
    let start = plaintext.len();
    plaintext.resize(start + padding_len, 0);
    OsRng.fill_bytes(&mut plaintext[start..]);
    let msg_key = compute_msg_key(auth_key, &plaintext, direction);
    let (aes_key, aes_iv) = derive_aes_key_iv(auth_key, &msg_key, direction);
    let encrypted = aes_ige_encrypt(&plaintext, &aes_key, &aes_iv)?;
    let mut frame = Vec::with_capacity(EXTERNAL_HEADER_LEN + encrypted.len());
    frame.extend_from_slice(&auth_key_id(auth_key));
    frame.extend_from_slice(&msg_key);
    frame.extend_from_slice(&encrypted);
    Ok(frame)
}

pub fn decrypt(auth_key: &[u8; AUTH_KEY_LEN], frame: &[u8], direction: Direction) -> Result<Message> {
    if frame.len() < EXTERNAL_HEADER_LEN + 48 || (frame.len() - EXTERNAL_HEADER_LEN) % 16 != 0 { return Err(Error::Invalid("invalid MTProto frame length")); }
    if auth_key_id(auth_key).ct_eq(&frame[..8]).unwrap_u8() != 1 { return Err(Error::Invalid("unknown auth key")); }
    let msg_key: [u8; 16] = frame[8..24].try_into().map_err(|_| Error::Invalid("invalid msg_key"))?;
    let (aes_key, aes_iv) = derive_aes_key_iv(auth_key, &msg_key, direction);
    let plaintext = aes_ige_decrypt(&frame[24..], &aes_key, &aes_iv)?;
    if compute_msg_key(auth_key, &plaintext, direction).ct_eq(&msg_key).unwrap_u8() != 1 { return Err(Error::Invalid("msg_key mismatch")); }
    if plaintext.len() < INTERNAL_HEADER_LEN { return Err(Error::Invalid("truncated internal header")); }
    let server_salt = i64::from_le_bytes(plaintext[0..8].try_into().map_err(|_| Error::Invalid("server salt"))?);
    let session_id = i64::from_le_bytes(plaintext[8..16].try_into().map_err(|_| Error::Invalid("session id"))?);
    let message_id = i64::from_le_bytes(plaintext[16..24].try_into().map_err(|_| Error::Invalid("message id"))?);
    let sequence_no = i32::from_le_bytes(plaintext[24..28].try_into().map_err(|_| Error::Invalid("sequence"))?);
    let body_len = i32::from_le_bytes(plaintext[28..32].try_into().map_err(|_| Error::Invalid("body length"))?);
    if body_len < 0 { return Err(Error::Invalid("negative body length")); }
    let body_end = INTERNAL_HEADER_LEN.checked_add(body_len as usize).ok_or(Error::Invalid("body overflow"))?;
    if body_end > plaintext.len() { return Err(Error::Invalid("truncated message body")); }
    let padding = plaintext.len() - body_end;
    if !(MIN_PADDING..=MAX_PADDING).contains(&padding) { return Err(Error::Invalid("invalid MTProto padding")); }
    validate_message_id(message_id, direction)?;
    Ok(Message { server_salt, session_id, message_id, sequence_no, body: plaintext[INTERNAL_HEADER_LEN..body_end].to_vec() })
}

pub fn compute_msg_key(auth_key: &[u8; AUTH_KEY_LEN], plaintext: &[u8], direction: Direction) -> [u8; 16] {
    let x = direction.x(); let mut hash = Sha256::new(); hash.update(&auth_key[88 + x..120 + x]); hash.update(plaintext);
    hash.finalize()[8..24].try_into().expect("fixed digest range")
}

pub fn derive_aes_key_iv(auth_key: &[u8; AUTH_KEY_LEN], msg_key: &[u8; 16], direction: Direction) -> ([u8; 32], [u8; 32]) {
    let x = direction.x();
    let mut a = Sha256::new(); a.update(msg_key); a.update(&auth_key[x..x + 36]); let a = a.finalize();
    let mut b = Sha256::new(); b.update(&auth_key[40 + x..76 + x]); b.update(msg_key); let b = b.finalize();
    let mut key = [0; 32]; key[..8].copy_from_slice(&a[..8]); key[8..24].copy_from_slice(&b[8..24]); key[24..].copy_from_slice(&a[24..]);
    let mut iv = [0; 32]; iv[..8].copy_from_slice(&b[..8]); iv[8..24].copy_from_slice(&a[8..24]); iv[24..].copy_from_slice(&b[24..]);
    (key, iv)
}

fn aes_ige_encrypt(input: &[u8], key: &[u8; 32], iv: &[u8; 32]) -> Result<Vec<u8>> { crypt_ige(input, key, iv, true) }
fn aes_ige_decrypt(input: &[u8], key: &[u8; 32], iv: &[u8; 32]) -> Result<Vec<u8>> { crypt_ige(input, key, iv, false) }
fn crypt_ige(input: &[u8], key: &[u8; 32], iv: &[u8; 32], encrypting: bool) -> Result<Vec<u8>> {
    if input.len() % 16 != 0 { return Err(Error::Invalid("AES-IGE input is not block aligned")); }
    let cipher = Aes256::new_from_slice(key).map_err(|_| Error::Invalid("invalid AES key"))?;
    let mut previous_cipher = iv[..16].to_vec(); let mut previous_plain = iv[16..].to_vec(); let mut output = Vec::with_capacity(input.len());
    for block in input.chunks_exact(16) {
        let mut mixed = [0; 16];
        for index in 0..16 { mixed[index] = block[index] ^ if encrypting { previous_cipher[index] } else { previous_plain[index] }; }
        let mut transformed = GenericArray::clone_from_slice(&mixed);
        if encrypting { cipher.encrypt_block(&mut transformed); } else { cipher.decrypt_block(&mut transformed); }
        let mut result = [0; 16];
        for index in 0..16 { result[index] = transformed[index] ^ if encrypting { previous_plain[index] } else { previous_cipher[index] }; }
        output.extend_from_slice(&result);
        if encrypting { previous_cipher = result.to_vec(); previous_plain = block.to_vec(); } else { previous_plain = result.to_vec(); previous_cipher = block.to_vec(); }
    }
    Ok(output)
}

fn padding_length(length: usize) -> usize { let mut padding = 16 - length % 16; if padding < MIN_PADDING { padding += 16; } padding }
pub fn validate_message_id(id: i64, direction: Direction) -> Result<()> { let modulo = id.rem_euclid(4); let valid = match direction { Direction::ClientToServer => modulo == 0, Direction::ServerToClient => modulo == 1 || modulo == 3 }; if valid { Ok(()) } else { Err(Error::Invalid("invalid message id parity")) } }

pub fn abridged_encode(payload: &[u8]) -> Result<Vec<u8>> {
    if payload.len() % 4 != 0 { return Err(Error::Invalid("abridged payload must be word aligned")); }
    let words = payload.len() / 4; let mut output = Vec::with_capacity(payload.len() + 4);
    if words < 0x7f { output.push(words as u8); } else { if words > 0xff_ffff { return Err(Error::Invalid("abridged payload too large")); } output.push(0x7f); output.extend_from_slice(&(words as u32).to_le_bytes()[..3]); }
    output.extend_from_slice(payload); Ok(output)
}

pub fn abridged_decode(input: &[u8]) -> Result<(Vec<u8>, usize)> {
    let Some(&first) = input.first() else {
        return Err(Error::Invalid("truncated abridged header"));
    };
    let (words, header_len) = if first < 0x7f {
        (usize::from(first), 1usize)
    } else {
        if input.len() < 4 {
            return Err(Error::Invalid("truncated abridged header"));
        }
        let words = u32::from_le_bytes([input[1], input[2], input[3], 0]) as usize;
        (words, 4usize)
    };
    if words == 0 {
        return Err(Error::Invalid("empty abridged payload"));
    }
    let payload_len = words
        .checked_mul(4)
        .ok_or(Error::Invalid("abridged payload overflow"))?;
    let end = header_len
        .checked_add(payload_len)
        .ok_or(Error::Invalid("abridged frame overflow"))?;
    if input.len() < end {
        return Err(Error::Invalid("truncated abridged payload"));
    }
    Ok((input[header_len..end].to_vec(), end))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn key() -> [u8; 256] { let mut key = [0; 256]; for (index, value) in key.iter_mut().enumerate() { *value = index as u8; } key }
    #[test] fn round_trip_both_directions() { let key = key(); for (direction, id) in [(Direction::ClientToServer, 0x1_0000_0000), (Direction::ServerToClient, 0x1_0000_0001)] { let message = Message { server_salt: 7, session_id: 9, message_id: id, sequence_no: 1, body: b"test".to_vec() }; let frame = encrypt(&key, &message, direction).unwrap(); assert_eq!(decrypt(&key, &frame, direction).unwrap(), message); } }
    #[test] fn generated_ids_are_monotonic_and_directional() { for direction in [Direction::ClientToServer, Direction::ServerToClient] { let generator = MessageIdGenerator::new(direction); let first = generator.next_id().unwrap(); let second = generator.next_id().unwrap(); assert!(second > first); validate_message_id(first, direction).unwrap(); validate_message_id(second, direction).unwrap(); } }
    #[test] fn tampering_is_rejected() { let key = key(); let message = Message { server_salt: 1, session_id: 2, message_id: 0x1_0000_0000, sequence_no: 1, body: vec![1, 2, 3] }; let mut frame = encrypt(&key, &message, Direction::ClientToServer).unwrap(); *frame.last_mut().unwrap() ^= 1; assert!(decrypt(&key, &frame, Direction::ClientToServer).is_err()); }
    #[test] fn abridged_transport_encodes_word_count() { assert_eq!(abridged_encode(&[0; 8]).unwrap(), vec![2, 0, 0, 0, 0, 0, 0, 0, 0]); }
    #[test] fn abridged_transport_round_trip() {
        for payload in [vec![7; 8], vec![9; 508]] {
            let encoded = abridged_encode(&payload).unwrap();
            let (decoded, consumed) = abridged_decode(&encoded).unwrap();
            assert_eq!(decoded, payload);
            assert_eq!(consumed, encoded.len());
        }
    }
    #[test] fn abridged_transport_rejects_truncation_and_empty_frames() {
        assert!(abridged_decode(&[]).is_err());
        assert!(abridged_decode(&[0]).is_err());
        assert!(abridged_decode(&[2, 0, 0, 0]).is_err());
        assert!(abridged_decode(&[0x7f, 2, 0]).is_err());
    }
}
