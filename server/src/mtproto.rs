//! MTProto 2.0 encrypted-message codec.
//!
//! This module implements the MTProto 2.0 established-session wire format used by TEKtalk:
//! auth_key_id, msg_key, SHA-256 KDF, AES-256-IGE, internal header and padding.
//! Authorization-key negotiation is intentionally outside this module.
use aes::{cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray}, Aes256};
use rand::{rngs::OsRng, RngCore};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::error::{AppError, AppResult};

pub const AUTH_KEY_LEN: usize = 256;
pub const EXTERNAL_HEADER_LEN: usize = 24;
pub const INTERNAL_HEADER_LEN: usize = 32;
const MIN_PADDING: usize = 12;
const MAX_PADDING: usize = 1024;

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

impl Direction { fn x(self) -> usize { match self { Self::ClientToServer => 0, Self::ServerToClient => 8 } } }

pub fn auth_key_id(auth_key: &[u8; AUTH_KEY_LEN]) -> [u8; 8] {
    let digest = Sha1::digest(auth_key);
    digest[12..20].try_into().expect("fixed digest range")
}

pub fn encrypt(auth_key: &[u8; AUTH_KEY_LEN], message: &Message, direction: Direction) -> AppResult<Vec<u8>> {
    validate_message_id(message.message_id, direction)?;
    if message.body.len() > i32::MAX as usize { return Err(AppError::Invalid("message body too large")); }
    let mut plaintext = Vec::with_capacity(INTERNAL_HEADER_LEN + message.body.len() + 32);
    plaintext.extend_from_slice(&message.server_salt.to_le_bytes());
    plaintext.extend_from_slice(&message.session_id.to_le_bytes());
    plaintext.extend_from_slice(&message.message_id.to_le_bytes());
    plaintext.extend_from_slice(&message.sequence_no.to_le_bytes());
    plaintext.extend_from_slice(&(message.body.len() as i32).to_le_bytes());
    plaintext.extend_from_slice(&message.body);
    let padding_len = padding_length(plaintext.len());
    let start = plaintext.len(); plaintext.resize(start + padding_len, 0); OsRng.fill_bytes(&mut plaintext[start..]);
    let msg_key = compute_msg_key(auth_key, &plaintext, direction);
    let (aes_key, aes_iv) = derive_aes_key_iv(auth_key, &msg_key, direction);
    let encrypted = aes_ige_encrypt(&plaintext, &aes_key, &aes_iv)?;
    let mut frame = Vec::with_capacity(EXTERNAL_HEADER_LEN + encrypted.len());
    frame.extend_from_slice(&auth_key_id(auth_key)); frame.extend_from_slice(&msg_key); frame.extend_from_slice(&encrypted);
    Ok(frame)
}

pub fn decrypt(auth_key: &[u8; AUTH_KEY_LEN], frame: &[u8], direction: Direction) -> AppResult<Message> {
    if frame.len() < EXTERNAL_HEADER_LEN + 48 || (frame.len() - EXTERNAL_HEADER_LEN) % 16 != 0 { return Err(AppError::Invalid("invalid MTProto frame length")); }
    if auth_key_id(auth_key).ct_eq(&frame[..8]).unwrap_u8() != 1 { return Err(AppError::Invalid("unknown auth key")); }
    let msg_key: [u8; 16] = frame[8..24].try_into().unwrap();
    let (aes_key, aes_iv) = derive_aes_key_iv(auth_key, &msg_key, direction);
    let plaintext = aes_ige_decrypt(&frame[24..], &aes_key, &aes_iv)?;
    if compute_msg_key(auth_key, &plaintext, direction).ct_eq(&msg_key).unwrap_u8() != 1 { return Err(AppError::Invalid("msg_key mismatch")); }
    if plaintext.len() < INTERNAL_HEADER_LEN { return Err(AppError::Invalid("truncated internal header")); }
    let server_salt=i64::from_le_bytes(plaintext[0..8].try_into().unwrap());
    let session_id=i64::from_le_bytes(plaintext[8..16].try_into().unwrap());
    let message_id=i64::from_le_bytes(plaintext[16..24].try_into().unwrap());
    let sequence_no=i32::from_le_bytes(plaintext[24..28].try_into().unwrap());
    let body_len=i32::from_le_bytes(plaintext[28..32].try_into().unwrap());
    if body_len < 0 { return Err(AppError::Invalid("negative body length")); }
    let body_end=INTERNAL_HEADER_LEN.checked_add(body_len as usize).ok_or(AppError::Invalid("body overflow"))?;
    if body_end > plaintext.len() { return Err(AppError::Invalid("truncated message body")); }
    let padding=plaintext.len()-body_end;
    if !(MIN_PADDING..=MAX_PADDING).contains(&padding) { return Err(AppError::Invalid("invalid MTProto padding")); }
    validate_message_id(message_id,direction)?;
    Ok(Message{server_salt,session_id,message_id,sequence_no,body:plaintext[INTERNAL_HEADER_LEN..body_end].to_vec()})
}

pub fn compute_msg_key(auth_key:&[u8;AUTH_KEY_LEN],plaintext:&[u8],direction:Direction)->[u8;16]{let x=direction.x();let mut h=Sha256::new();h.update(&auth_key[88+x..120+x]);h.update(plaintext);let d=h.finalize();d[8..24].try_into().unwrap()}

pub fn derive_aes_key_iv(auth_key:&[u8;AUTH_KEY_LEN],msg_key:&[u8;16],direction:Direction)->([u8;32],[u8;32]){let x=direction.x();let mut a=Sha256::new();a.update(msg_key);a.update(&auth_key[x..x+36]);let a=a.finalize();let mut b=Sha256::new();b.update(&auth_key[40+x..76+x]);b.update(msg_key);let b=b.finalize();let mut key=[0u8;32];key[..8].copy_from_slice(&a[..8]);key[8..24].copy_from_slice(&b[8..24]);key[24..].copy_from_slice(&a[24..32]);let mut iv=[0u8;32];iv[..8].copy_from_slice(&b[..8]);iv[8..24].copy_from_slice(&a[8..24]);iv[24..].copy_from_slice(&b[24..32]);(key,iv)}

fn aes_ige_encrypt(input:&[u8],key:&[u8;32],iv:&[u8;32])->AppResult<Vec<u8>>{if input.len()%16!=0{return Err(AppError::Invalid("AES-IGE input is not block aligned"));}let cipher=Aes256::new_from_slice(key).map_err(|_|AppError::Invalid("invalid AES key"))?;let mut prev_c=iv[..16].to_vec();let mut prev_p=iv[16..].to_vec();let mut out=Vec::with_capacity(input.len());for p in input.chunks_exact(16){let mut x=[0u8;16];for i in 0..16{x[i]=p[i]^prev_c[i]}let mut block=GenericArray::clone_from_slice(&x);cipher.encrypt_block(&mut block);let mut c=[0u8;16];for i in 0..16{c[i]=block[i]^prev_p[i]}out.extend_from_slice(&c);prev_c=c.to_vec();prev_p=p.to_vec();}Ok(out)}
fn aes_ige_decrypt(input:&[u8],key:&[u8;32],iv:&[u8;32])->AppResult<Vec<u8>>{if input.len()%16!=0{return Err(AppError::Invalid("AES-IGE input is not block aligned"));}let cipher=Aes256::new_from_slice(key).map_err(|_|AppError::Invalid("invalid AES key"))?;let mut prev_c=iv[..16].to_vec();let mut prev_p=iv[16..].to_vec();let mut out=Vec::with_capacity(input.len());for c in input.chunks_exact(16){let mut x=[0u8;16];for i in 0..16{x[i]=c[i]^prev_p[i]}let mut block=GenericArray::clone_from_slice(&x);cipher.decrypt_block(&mut block);let mut p=[0u8;16];for i in 0..16{p[i]=block[i]^prev_c[i]}out.extend_from_slice(&p);prev_c=c.to_vec();prev_p=p.to_vec();}Ok(out)}
fn padding_length(len:usize)->usize{let mut n=16-(len%16);if n<MIN_PADDING{n+=16}n}
fn validate_message_id(id:i64,direction:Direction)->AppResult<()>{let modulo=id.rem_euclid(4);let valid=match direction{Direction::ClientToServer=>modulo==0,Direction::ServerToClient=>modulo==1||modulo==3};if !valid{return Err(AppError::Invalid("invalid message id parity"));}Ok(())}

pub fn abridged_encode(payload:&[u8])->AppResult<Vec<u8>>{if payload.len()%4!=0{return Err(AppError::Invalid("abridged payload must be word aligned"));}let words=payload.len()/4;let mut out=Vec::with_capacity(payload.len()+4);if words<0x7f{out.push(words as u8)}else{if words>0xff_ffff{return Err(AppError::Invalid("abridged payload too large"));}out.push(0x7f);out.extend_from_slice(&(words as u32).to_le_bytes()[..3]);}out.extend_from_slice(payload);Ok(out)}

#[cfg(test)] mod tests{use super::*;fn key()->[u8;256]{let mut k=[0u8;256];for(i,v)in k.iter_mut().enumerate(){*v=i as u8}k}#[test]fn roundtrip_both_directions(){let k=key();for(d,id)in[(Direction::ClientToServer,0x100000000i64),(Direction::ServerToClient,0x100000001i64)]{let m=Message{server_salt:7,session_id:9,message_id:id,sequence_no:1,body:b"test body".to_vec()};let f=encrypt(&k,&m,d).unwrap();let got=decrypt(&k,&f,d).unwrap();assert_eq!(got,m)}}#[test]fn tamper_is_rejected(){let k=key();let m=Message{server_salt:1,session_id:2,message_id:0x100000000,sequence_no:1,body:vec![1,2,3,4]};let mut f=encrypt(&k,&m,Direction::ClientToServer).unwrap();*f.last_mut().unwrap()^=1;assert!(decrypt(&k,&f,Direction::ClientToServer).is_err())}#[test]fn abridged_short(){assert_eq!(abridged_encode(&[0;8]).unwrap(),vec![2,0,0,0,0,0,0,0,0])}}
