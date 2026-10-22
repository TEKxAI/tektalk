//! Adaptive file and realtime-media transport primitives.
//!
//! MTProto remains the chat/control protocol. This module prepares encrypted,
//! resumable file chunks for reliable QUIC streams and computes conservative
//! media profiles for QUIC DATAGRAM adapters implemented by each platform.

use aes_gcm_siv::{
    aead::{Aead, Payload},
    Aes256GcmSiv, KeyInit, Nonce,
};
use hkdf::Hkdf;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::secret_chat::SessionKey;

const LABEL: &[u8] = b"TEKtalk Adaptive Transport v1";
pub const DEFAULT_CHUNK_SIZE: u32 = 256 * 1024;
pub const MAX_CHUNK_SIZE: u32 = 4 * 1024 * 1024;
pub const MAX_PARALLEL_STREAMS: u8 = 8;

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum Error {
    #[error("invalid transfer manifest")]
    InvalidManifest,
    #[error("invalid chunk")]
    InvalidChunk,
    #[error("authenticated decryption failed")]
    Authentication,
    #[error("file hash mismatch")]
    HashMismatch,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransferManifest {
    pub file_id: String,
    pub conversation_id: String,
    pub epoch: u64,
    pub content_length: u64,
    pub chunk_size: u32,
    pub chunk_count: u32,
    pub plaintext_sha256: [u8; 32],
}

impl TransferManifest {
    pub fn new(
        file_id: impl Into<String>,
        conversation_id: impl Into<String>,
        epoch: u64,
        plaintext: &[u8],
        chunk_size: u32,
    ) -> Result<Self> {
        if chunk_size == 0 || chunk_size > MAX_CHUNK_SIZE {
            return Err(Error::InvalidManifest);
        }
        let count = plaintext.len().div_ceil(chunk_size as usize);
        let chunk_count = u32::try_from(count).map_err(|_| Error::InvalidManifest)?;
        if plaintext.is_empty() || chunk_count == 0 {
            return Err(Error::InvalidManifest);
        }
        Ok(Self {
            file_id: file_id.into(),
            conversation_id: conversation_id.into(),
            epoch,
            content_length: plaintext.len() as u64,
            chunk_size,
            chunk_count,
            plaintext_sha256: Sha256::digest(plaintext).into(),
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.file_id.is_empty()
            || self.conversation_id.is_empty()
            || self.epoch == 0
            || self.content_length == 0
            || self.chunk_size == 0
            || self.chunk_size > MAX_CHUNK_SIZE
            || self.chunk_count == 0
            || u64::from(self.chunk_count)
                != self.content_length.div_ceil(u64::from(self.chunk_size))
        {
            return Err(Error::InvalidManifest);
        }
        Ok(())
    }
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct TransferKey([u8; 32]);

impl TransferKey {
    pub fn derive(session: &SessionKey, manifest: &TransferManifest) -> Result<Self> {
        manifest.validate()?;
        let hkdf = Hkdf::<Sha256>::new(Some(&manifest.plaintext_sha256), session.expose_for_key_wrap());
        let mut key = [0u8; 32];
        let mut info = Vec::from(LABEL);
        info.extend_from_slice(manifest.conversation_id.as_bytes());
        info.extend_from_slice(manifest.file_id.as_bytes());
        info.extend_from_slice(&manifest.epoch.to_be_bytes());
        hkdf.expand(&info, &mut key).map_err(|_| Error::InvalidManifest)?;
        info.zeroize();
        Ok(Self(key))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EncryptedChunk {
    pub index: u32,
    pub plaintext_length: u32,
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
}

pub fn encrypt_chunk(
    key: &TransferKey,
    manifest: &TransferManifest,
    index: u32,
    plaintext: &[u8],
) -> Result<EncryptedChunk> {
    validate_chunk_shape(manifest, index, plaintext.len())?;
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    let aad = chunk_aad(manifest, index, plaintext.len() as u32);
    let cipher = Aes256GcmSiv::new_from_slice(&key.0).map_err(|_| Error::InvalidChunk)?;
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), Payload { msg: plaintext, aad: &aad })
        .map_err(|_| Error::Authentication)?;
    Ok(EncryptedChunk { index, plaintext_length: plaintext.len() as u32, nonce, ciphertext })
}

pub fn decrypt_chunk(
    key: &TransferKey,
    manifest: &TransferManifest,
    chunk: &EncryptedChunk,
) -> Result<Vec<u8>> {
    validate_chunk_shape(manifest, chunk.index, chunk.plaintext_length as usize)?;
    let aad = chunk_aad(manifest, chunk.index, chunk.plaintext_length);
    let cipher = Aes256GcmSiv::new_from_slice(&key.0).map_err(|_| Error::InvalidChunk)?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&chunk.nonce), Payload { msg: &chunk.ciphertext, aad: &aad })
        .map_err(|_| Error::Authentication)?;
    if plaintext.len() != chunk.plaintext_length as usize {
        return Err(Error::InvalidChunk);
    }
    Ok(plaintext)
}

pub fn verify_reassembled(manifest: &TransferManifest, plaintext: &[u8]) -> Result<()> {
    if plaintext.len() as u64 != manifest.content_length
        || <[u8; 32]>::from(Sha256::digest(plaintext)) != manifest.plaintext_sha256
    {
        return Err(Error::HashMismatch);
    }
    Ok(())
}

fn validate_chunk_shape(manifest: &TransferManifest, index: u32, length: usize) -> Result<()> {
    manifest.validate()?;
    if index >= manifest.chunk_count || length == 0 || length > manifest.chunk_size as usize {
        return Err(Error::InvalidChunk);
    }
    let expected = if index + 1 == manifest.chunk_count {
        (manifest.content_length - u64::from(index) * u64::from(manifest.chunk_size)) as usize
    } else {
        manifest.chunk_size as usize
    };
    if length != expected { return Err(Error::InvalidChunk); }
    Ok(())
}

fn chunk_aad(manifest: &TransferManifest, index: u32, length: u32) -> Vec<u8> {
    let mut aad = Vec::from(LABEL);
    aad.extend_from_slice(manifest.conversation_id.as_bytes());
    aad.extend_from_slice(manifest.file_id.as_bytes());
    aad.extend_from_slice(&manifest.epoch.to_be_bytes());
    aad.extend_from_slice(&index.to_be_bytes());
    aad.extend_from_slice(&manifest.chunk_count.to_be_bytes());
    aad.extend_from_slice(&length.to_be_bytes());
    aad.extend_from_slice(&manifest.plaintext_sha256);
    aad
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResumeBitmap { completed: BTreeSet<u32> }

impl ResumeBitmap {
    pub fn mark_complete(&mut self, index: u32, manifest: &TransferManifest) -> Result<()> {
        if index >= manifest.chunk_count { return Err(Error::InvalidChunk); }
        self.completed.insert(index);
        Ok(())
    }

    pub fn missing(&self, manifest: &TransferManifest) -> Vec<u32> {
        (0..manifest.chunk_count).filter(|index| !self.completed.contains(index)).collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkSample {
    pub available_kbps: u32,
    pub rtt_ms: u32,
    pub jitter_ms: u32,
    pub loss_per_mille: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StreamPlan { pub parallel_streams: u8 }

impl StreamPlan {
    pub fn for_network(sample: NetworkSample) -> Self {
        let streams = if sample.loss_per_mille >= 80 || sample.rtt_ms >= 500 { 1 }
        else if sample.available_kbps < 2_000 || sample.loss_per_mille >= 30 { 2 }
        else if sample.available_kbps < 20_000 { 4 }
        else { MAX_PARALLEL_STREAMS };
        Self { parallel_streams: streams }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MediaProfile {
    pub target_bitrate_kbps: u32,
    pub fec_percent: u8,
    pub retransmission_window_ms: u16,
}

impl MediaProfile {
    pub fn adapt(sample: NetworkSample) -> Self {
        let loss = u32::from(sample.loss_per_mille);
        let reserve = if loss >= 100 { 45 } else if loss >= 50 { 30 } else { 15 };
        let target = sample.available_kbps.saturating_mul(100 - reserve) / 100;
        let fec_percent = if loss >= 100 { 30 } else if loss >= 50 { 20 } else if loss >= 20 { 10 } else { 0 };
        let retransmission_window_ms = if sample.rtt_ms + sample.jitter_ms <= 120 { 120 }
            else if sample.rtt_ms <= 250 { 80 } else { 0 };
        Self { target_bitrate_kbps: target.max(24), fec_percent, retransmission_window_ms }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secret_chat::{create_session, ConversationKind, DeviceKeyPair, SecretChatPolicy};

    fn session() -> SessionKey {
        let recipient = DeviceKeyPair::generate("bob", "phone");
        let policy = SecretChatPolicy::opt_in(ConversationKind::OneToOne);
        let (envelope, sender) = create_session(&recipient.bundle, &policy, "conversation").unwrap();
        let receiver = recipient.open_session(&envelope, &policy).unwrap();
        assert_eq!(sender.expose_for_key_wrap(), receiver.expose_for_key_wrap());
        sender
    }

    #[test]
    fn encrypted_chunks_reassemble_out_of_order() {
        let data = vec![42u8; 700_001];
        let manifest = TransferManifest::new("file", "conversation", 1, &data, 256 * 1024).unwrap();
        let key = TransferKey::derive(&session(), &manifest).unwrap();
        let mut chunks: Vec<_> = data.chunks(manifest.chunk_size as usize).enumerate()
            .map(|(index, bytes)| encrypt_chunk(&key, &manifest, index as u32, bytes).unwrap()).collect();
        chunks.reverse();
        let mut opened: Vec<_> = chunks.iter().map(|chunk| (chunk.index, decrypt_chunk(&key, &manifest, chunk).unwrap())).collect();
        opened.sort_by_key(|(index, _)| *index);
        let rebuilt: Vec<u8> = opened.into_iter().flat_map(|(_, bytes)| bytes).collect();
        verify_reassembled(&manifest, &rebuilt).unwrap();
    }

    #[test]
    fn metadata_tampering_is_rejected() {
        let data = vec![7u8; 32];
        let manifest = TransferManifest::new("file", "conversation", 1, &data, 32).unwrap();
        let key = TransferKey::derive(&session(), &manifest).unwrap();
        let mut chunk = encrypt_chunk(&key, &manifest, 0, &data).unwrap();
        chunk.ciphertext[0] ^= 1;
        assert_eq!(decrypt_chunk(&key, &manifest, &chunk), Err(Error::Authentication));
    }

    #[test]
    fn resume_and_network_profiles_are_bounded() {
        let data = vec![1u8; 10];
        let manifest = TransferManifest::new("file", "conversation", 1, &data, 4).unwrap();
        let mut resume = ResumeBitmap::default();
        resume.mark_complete(1, &manifest).unwrap();
        assert_eq!(resume.missing(&manifest), vec![0, 2]);
        let weak = NetworkSample { available_kbps: 100, rtt_ms: 600, jitter_ms: 100, loss_per_mille: 120 };
        assert_eq!(StreamPlan::for_network(weak).parallel_streams, 1);
        let profile = MediaProfile::adapt(weak);
        assert_eq!(profile.fec_percent, 30);
        assert_eq!(profile.retransmission_window_ms, 0);
    }
}
