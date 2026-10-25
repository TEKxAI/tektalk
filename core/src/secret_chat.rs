//! Opt-in, multi-device end-to-end encryption for one-to-one and group chats.
//!
//! The server transports [`DeviceBundle`] and opaque ciphertext only. Content
//! keys are established with a hybrid X25519 + ML-KEM-768 exchange and then
//! used with AES-256-GCM-SIV. Hybrid is mandatory once enabled; there is no
//! classical-only downgrade path in this module.

use aes_gcm_siv::{
    aead::{Aead, Payload},
    Aes256GcmSiv, KeyInit, Nonce,
};
use hkdf::Hkdf;
use ml_kem::{
    kem::{Decapsulate, Encapsulate, Kem, KeyExport, TryKeyInit},
    MlKem768,
};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::{Zeroize, ZeroizeOnDrop};

pub const SUITE: &str = "TEKTALK_X25519_MLKEM768_AES256GCMSIV_SHA256";
const PROTOCOL_LABEL: &[u8] = b"TEKtalk Secret Chat v1";

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum Error {
    #[error("secret chat was not explicitly enabled")]
    NotOptedIn,
    #[error("peer does not support the mandatory hybrid suite")]
    DowngradeRejected,
    #[error("invalid key material")]
    InvalidKey,
    #[error("membership epoch is stale")]
    StaleEpoch,
    #[error("message counter was replayed")]
    Replay,
    #[error("authenticated decryption failed")]
    Authentication,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConversationKind {
    OneToOne,
    Group,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SecretChatPolicy {
    pub enabled: bool,
    pub kind: ConversationKind,
    pub required_suite: String,
    pub epoch: u64,
}

impl SecretChatPolicy {
    pub fn opt_in(kind: ConversationKind) -> Self {
        Self { enabled: true, kind, required_suite: SUITE.into(), epoch: 1 }
    }

    pub fn require_hybrid(&self, advertised_suites: &[String]) -> Result<()> {
        if !self.enabled { return Err(Error::NotOptedIn); }
        if self.required_suite != SUITE || !advertised_suites.iter().any(|suite| suite == SUITE) {
            return Err(Error::DowngradeRejected);
        }
        Ok(())
    }

    pub fn membership_changed(&mut self) {
        self.epoch = self.epoch.saturating_add(1);
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeviceBundle {
    pub user_id: String,
    pub device_id: String,
    pub x25519_public: [u8; 32],
    pub ml_kem_768_public: Vec<u8>,
    pub suites: Vec<String>,
}

pub struct DeviceKeyPair {
    pub bundle: DeviceBundle,
    x25519_secret: StaticSecret,
    ml_kem_secret: <MlKem768 as Kem>::DecapsulationKey,
}

impl DeviceKeyPair {
    pub(crate) fn export_identity(&self) -> (DeviceBundle, [u8; 32], Vec<u8>) {
        (self.bundle.clone(), self.x25519_secret.to_bytes(), self.ml_kem_secret.to_bytes().as_slice().to_vec())
    }

    pub(crate) fn restore_identity(bundle: DeviceBundle, x25519: [u8; 32], seed: &[u8]) -> Result<Self> {
        let x25519_secret = StaticSecret::from(x25519);
        let seed = ml_kem::Seed::try_from(seed).map_err(|_| Error::InvalidKey)?;
        let ml_kem_secret = <MlKem768 as Kem>::DecapsulationKey::from_seed(seed);
        if PublicKey::from(&x25519_secret).to_bytes() != bundle.x25519_public
            || ml_kem_secret.encapsulation_key().to_bytes().as_slice() != bundle.ml_kem_768_public.as_slice() {
            return Err(Error::InvalidKey);
        }
        Ok(Self { bundle, x25519_secret, ml_kem_secret })
    }

    pub fn generate(user_id: impl Into<String>, device_id: impl Into<String>) -> Self {
        let x25519_secret = StaticSecret::random_from_rng(OsRng);
        let x25519_public = PublicKey::from(&x25519_secret).to_bytes();
        let (ml_kem_secret, ml_kem_public) = MlKem768::generate_keypair();
        Self {
            bundle: DeviceBundle {
                user_id: user_id.into(),
                device_id: device_id.into(),
                x25519_public,
                ml_kem_768_public: ml_kem_public.to_bytes().as_slice().to_vec(),
                suites: vec![SUITE.into()],
            },
            x25519_secret,
            ml_kem_secret,
        }
    }

    pub fn open_session(&self, envelope: &SessionEnvelope, policy: &SecretChatPolicy) -> Result<SessionKey> {
        policy.require_hybrid(&self.bundle.suites)?;
        if envelope.suite != SUITE { return Err(Error::DowngradeRejected); }
        let ciphertext = ml_kem::Ciphertext::<MlKem768>::try_from(envelope.ml_kem_ciphertext.as_slice())
            .map_err(|_| Error::InvalidKey)?;
        let pq_secret = self.ml_kem_secret.decapsulate(&ciphertext);
        let peer_public = PublicKey::from(envelope.ephemeral_x25519_public);
        let classical_secret = self.x25519_secret.diffie_hellman(&peer_public);
        derive_session_key(
            classical_secret.as_bytes(),
            pq_secret.as_slice(),
            &envelope.transcript_hash,
            policy.epoch,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionEnvelope {
    pub suite: String,
    pub recipient_device_id: String,
    pub ephemeral_x25519_public: [u8; 32],
    pub ml_kem_ciphertext: Vec<u8>,
    pub transcript_hash: [u8; 32],
}

pub fn create_session(
    recipient: &DeviceBundle,
    policy: &SecretChatPolicy,
    conversation_id: &str,
) -> Result<(SessionEnvelope, SessionKey)> {
    policy.require_hybrid(&recipient.suites)?;
    let recipient_pq = <MlKem768 as Kem>::EncapsulationKey::new_from_slice(
        recipient.ml_kem_768_public.as_slice(),
    ).map_err(|_| Error::InvalidKey)?;
    let (pq_ciphertext, pq_secret) = recipient_pq.encapsulate();
    let ephemeral_secret = StaticSecret::random_from_rng(OsRng);
    let ephemeral_public = PublicKey::from(&ephemeral_secret).to_bytes();
    let recipient_x25519 = PublicKey::from(recipient.x25519_public);
    let classical_secret = ephemeral_secret.diffie_hellman(&recipient_x25519);
    let transcript_hash = transcript_hash(conversation_id, recipient, policy.epoch, &ephemeral_public);
    let session = derive_session_key(
        classical_secret.as_bytes(),
        pq_secret.as_slice(),
        &transcript_hash,
        policy.epoch,
    )?;
    Ok((SessionEnvelope {
        suite: SUITE.into(),
        recipient_device_id: recipient.device_id.clone(),
        ephemeral_x25519_public: ephemeral_public,
        ml_kem_ciphertext: pq_ciphertext.as_slice().to_vec(),
        transcript_hash,
    }, session))
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SessionKey([u8; 32]);

impl SessionKey {
    pub fn expose_for_key_wrap(&self) -> &[u8; 32] { &self.0 }
}

fn derive_session_key(classical: &[u8], pq: &[u8], transcript: &[u8; 32], epoch: u64) -> Result<SessionKey> {
    let mut ikm = Vec::with_capacity(classical.len() + pq.len());
    ikm.extend_from_slice(classical);
    ikm.extend_from_slice(pq);
    let hkdf = Hkdf::<Sha256>::new(Some(transcript), &ikm);
    let mut output = [0u8; 32];
    let mut info = Vec::from(PROTOCOL_LABEL);
    info.extend_from_slice(&epoch.to_be_bytes());
    hkdf.expand(&info, &mut output).map_err(|_| Error::InvalidKey)?;
    ikm.zeroize();
    Ok(SessionKey(output))
}

fn transcript_hash(conversation_id: &str, recipient: &DeviceBundle, epoch: u64, ephemeral: &[u8; 32]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(PROTOCOL_LABEL);
    digest.update(SUITE.as_bytes());
    digest.update(conversation_id.as_bytes());
    digest.update(recipient.user_id.as_bytes());
    digest.update(recipient.device_id.as_bytes());
    digest.update(epoch.to_be_bytes());
    digest.update(ephemeral);
    digest.update(recipient.x25519_public);
    digest.update(&recipient.ml_kem_768_public);
    digest.finalize().into()
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub conversation_id: String,
    pub epoch: u64,
    pub sender_device_id: String,
    pub counter: u64,
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct SenderKey {
    key: [u8; 32],
    epoch: u64,
    next_counter: u64,
}

impl SenderKey {
    pub fn generate(epoch: u64) -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        Self { key, epoch, next_counter: 0 }
    }

    pub fn from_bytes(key: [u8; 32], epoch: u64) -> Self {
        Self { key, epoch, next_counter: 0 }
    }

    pub fn encrypt(&mut self, conversation_id: &str, sender_device_id: &str, plaintext: &[u8]) -> Result<EncryptedMessage> {
        let counter = self.next_counter;
        self.next_counter = self.next_counter.checked_add(1).ok_or(Error::Replay)?;
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        let aad = message_aad(conversation_id, self.epoch, sender_device_id, counter);
        let ciphertext = Aes256GcmSiv::new((&self.key).into())
            .encrypt(Nonce::from_slice(&nonce), Payload { msg: plaintext, aad: &aad })
            .map_err(|_| Error::Authentication)?;
        Ok(EncryptedMessage {
            conversation_id: conversation_id.into(), epoch: self.epoch,
            sender_device_id: sender_device_id.into(), counter, nonce, ciphertext,
        })
    }

    pub fn decrypt(&self, message: &EncryptedMessage, replay: &mut ReplayGuard) -> Result<Vec<u8>> {
        if message.epoch != self.epoch { return Err(Error::StaleEpoch); }
        let aad = message_aad(&message.conversation_id, message.epoch, &message.sender_device_id, message.counter);
        let plaintext = Aes256GcmSiv::new((&self.key).into())
            .decrypt(Nonce::from_slice(&message.nonce), Payload { msg: &message.ciphertext, aad: &aad })
            .map_err(|_| Error::Authentication)?;
        replay.accept(&message.conversation_id, &message.sender_device_id, message.epoch, message.counter)?;
        Ok(plaintext)
    }
}

fn message_aad(conversation_id: &str, epoch: u64, sender: &str, counter: u64) -> Vec<u8> {
    let mut aad = Vec::from(PROTOCOL_LABEL);
    aad.extend_from_slice(conversation_id.as_bytes());
    aad.extend_from_slice(&epoch.to_be_bytes());
    aad.extend_from_slice(sender.as_bytes());
    aad.extend_from_slice(&counter.to_be_bytes());
    aad
}

#[derive(Default)]
pub struct ReplayGuard(BTreeMap<(String, String, u64), BTreeSet<u64>>);

impl ReplayGuard {
    pub(crate) fn export(&self) -> Vec<(String, String, u64, Vec<u64>)> {
        self.0.iter().map(|((conversation, sender, epoch), counters)| (conversation.clone(), sender.clone(), *epoch, counters.iter().copied().collect())).collect()
    }

    pub(crate) fn restore(rows: Vec<(String, String, u64, Vec<u64>)>) -> Self {
        Self(rows.into_iter().map(|(conversation, sender, epoch, counters)| ((conversation, sender, epoch), counters.into_iter().collect())).collect())
    }

    fn accept(&mut self, conversation: &str, sender: &str, epoch: u64, counter: u64) -> Result<()> {
        if !self.0.entry((conversation.into(), sender.into(), epoch)).or_default().insert(counter) {
            return Err(Error::Replay);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WrappedSenderKey {
    pub recipient_device_id: String,
    pub envelope: SessionEnvelope,
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
    pub epoch: u64,
}

/// Wraps a fresh sender key for every device in a 1-1 or group membership.
/// Adding/removing a device must first increment `policy.epoch` and redistribute.
pub fn distribute_sender_key(
    conversation_id: &str,
    sender_key: &SenderKey,
    recipients: &[DeviceBundle],
    policy: &SecretChatPolicy,
) -> Result<Vec<WrappedSenderKey>> {
    if sender_key.epoch != policy.epoch { return Err(Error::StaleEpoch); }
    recipients.iter().map(|recipient| {
        let (envelope, session) = create_session(recipient, policy, conversation_id)?;
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        let aad = message_aad(conversation_id, policy.epoch, &recipient.device_id, 0);
        let ciphertext = Aes256GcmSiv::new(session.expose_for_key_wrap().into())
            .encrypt(Nonce::from_slice(&nonce), Payload { msg: &sender_key.key, aad: &aad })
            .map_err(|_| Error::Authentication)?;
        Ok(WrappedSenderKey { recipient_device_id: recipient.device_id.clone(), envelope, nonce, ciphertext, epoch: policy.epoch })
    }).collect()
}

pub fn unwrap_sender_key(
    conversation_id: &str,
    recipient: &DeviceKeyPair,
    wrapped: &WrappedSenderKey,
    policy: &SecretChatPolicy,
) -> Result<SenderKey> {
    if wrapped.epoch != policy.epoch { return Err(Error::StaleEpoch); }
    let session = recipient.open_session(&wrapped.envelope, policy)?;
    let aad = message_aad(conversation_id, policy.epoch, &recipient.bundle.device_id, 0);
    let plaintext = Aes256GcmSiv::new(session.expose_for_key_wrap().into())
        .decrypt(Nonce::from_slice(&wrapped.nonce), Payload { msg: &wrapped.ciphertext, aad: &aad })
        .map_err(|_| Error::Authentication)?;
    let key: [u8; 32] = plaintext.try_into().map_err(|_| Error::InvalidKey)?;
    Ok(SenderKey::from_bytes(key, policy.epoch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hybrid_session_matches_on_both_devices() {
        let bob = DeviceKeyPair::generate("bob", "bob-phone");
        let policy = SecretChatPolicy::opt_in(ConversationKind::OneToOne);
        let (envelope, alice_key) = create_session(&bob.bundle, &policy, "chat-1").unwrap();
        let bob_key = bob.open_session(&envelope, &policy).unwrap();
        assert_eq!(alice_key.expose_for_key_wrap(), bob_key.expose_for_key_wrap());
    }

    #[test]
    fn group_sender_key_reaches_every_device_and_rejects_replay() {
        let devices = [
            DeviceKeyPair::generate("bob", "bob-phone"),
            DeviceKeyPair::generate("bob", "bob-laptop"),
            DeviceKeyPair::generate("carol", "carol-phone"),
        ];
        let policy = SecretChatPolicy::opt_in(ConversationKind::Group);
        let mut sender = SenderKey::generate(policy.epoch);
        let bundles = devices.iter().map(|device| device.bundle.clone()).collect::<Vec<_>>();
        let wrapped = distribute_sender_key("group-1", &sender, &bundles, &policy).unwrap();
        let message = sender.encrypt("group-1", "alice-phone", b"secret hello").unwrap();
        for (device, envelope) in devices.iter().zip(&wrapped) {
            let receiver = unwrap_sender_key("group-1", device, envelope, &policy).unwrap();
            let mut replay = ReplayGuard::default();
            assert_eq!(receiver.decrypt(&message, &mut replay).unwrap(), b"secret hello");
            assert_eq!(receiver.decrypt(&message, &mut replay), Err(Error::Replay));
        }
    }

    #[test]
    fn rejects_downgrade_and_stale_membership_epoch() {
        let mut bob = DeviceKeyPair::generate("bob", "bob-phone");
        bob.bundle.suites.clear();
        let mut policy = SecretChatPolicy::opt_in(ConversationKind::Group);
        assert!(matches!(create_session(&bob.bundle, &policy, "group-1"), Err(Error::DowngradeRejected)));
        let sender = SenderKey::generate(policy.epoch);
        policy.membership_changed();
        assert!(matches!(distribute_sender_key("group-1", &sender, &[], &policy), Err(Error::StaleEpoch)));
    }

    #[test]
    fn aad_tampering_fails_closed() {
        let mut sender = SenderKey::generate(1);
        let mut message = sender.encrypt("group-1", "alice-phone", b"hello").unwrap();
        message.conversation_id = "other-group".into();
        let receiver = SenderKey::from_bytes(sender.key, 1);
        assert_eq!(receiver.decrypt(&message, &mut ReplayGuard::default()), Err(Error::Authentication));
    }
}
