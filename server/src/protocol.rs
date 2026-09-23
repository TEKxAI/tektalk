use chacha20poly1305::{aead::{Aead, Payload}, ChaCha20Poly1305, KeyInit, Nonce};
use crate::error::{AppError, AppResult};

pub const VERSION:u8=1; pub const HEADER_LEN:usize=32;
#[derive(Debug,Clone,Copy)] pub struct Header{pub kind:u8,pub flags:u16,pub session_id:u64,pub message_id:u64,pub sequence:u64,pub payload_len:u32}

impl Header{
 pub fn encode(&self)->[u8;HEADER_LEN]{let mut b=[0u8;HEADER_LEN];b[0]=VERSION;b[1]=self.kind;b[2..4].copy_from_slice(&self.flags.to_be_bytes());b[4..12].copy_from_slice(&self.session_id.to_be_bytes());b[12..20].copy_from_slice(&self.message_id.to_be_bytes());b[20..28].copy_from_slice(&self.sequence.to_be_bytes());b[28..32].copy_from_slice(&self.payload_len.to_be_bytes());b}
 pub fn decode(b:&[u8])->AppResult<Self>{if b.len()<HEADER_LEN||b[0]!=VERSION{return Err(AppError::Invalid("bad frame header"));}Ok(Self{kind:b[1],flags:u16::from_be_bytes(b[2..4].try_into().unwrap()),session_id:u64::from_be_bytes(b[4..12].try_into().unwrap()),message_id:u64::from_be_bytes(b[12..20].try_into().unwrap()),sequence:u64::from_be_bytes(b[20..28].try_into().unwrap()),payload_len:u32::from_be_bytes(b[28..32].try_into().unwrap())})}
}
pub fn decrypt(key:&[u8;32],frame:&[u8])->AppResult<(Header,Vec<u8>)>{let h=Header::decode(frame)?;let aad=&frame[..HEADER_LEN];let body=&frame[HEADER_LEN..];if body.len()!=h.payload_len as usize+16{return Err(AppError::Invalid("bad payload length"));}let cipher=ChaCha20Poly1305::new(key.into());let n=nonce(h.sequence,h.message_id);let p=cipher.decrypt(&n,Payload{msg:body,aad}).map_err(|_|AppError::Invalid("authentication failed"))?;Ok((h,p))}
pub fn encrypt(key:&[u8;32],mut h:Header,payload:&[u8])->AppResult<Vec<u8>>{h.payload_len=payload.len() as u32;let aad=h.encode();let cipher=ChaCha20Poly1305::new(key.into());let n=nonce(h.sequence,h.message_id);let body=cipher.encrypt(&n,Payload{msg:payload,aad:&aad}).map_err(|e|AppError::Internal(anyhow::anyhow!(e)))?;let mut out=aad.to_vec();out.extend(body);Ok(out)}
fn nonce(seq:u64,msg:u64)->Nonce{let mut n=[0u8;12];n[..8].copy_from_slice(&seq.to_be_bytes());n[8..].copy_from_slice(&(msg as u32).to_be_bytes());*Nonce::from_slice(&n)}

#[cfg(test)] mod tests{use super::*;#[test]fn round_trip(){let key=[7u8;32];let h=Header{kind:1,flags:0,session_id:2,message_id:3,sequence:4,payload_len:0};let f=encrypt(&key,h,b"hello").unwrap();let(_,p)=decrypt(&key,&f).unwrap();assert_eq!(p,b"hello");}}

