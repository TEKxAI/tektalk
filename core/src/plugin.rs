use std::collections::HashSet;

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustTier {
    ThirdParty,
    FirstParty,
    Host,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginManifest {
    pub schema_version: u32,
    pub id: String,
    pub version: String,
    pub entry_point: String,
    pub plugin_type: String,
    pub minimum_host_version: String,
    pub minimum_core_abi: u32,
    pub valdi_runtime: String,
    pub capabilities: HashSet<String>,
    pub network_allowlist: Vec<String>,
    pub storage_namespace: String,
    pub trust_tier: TrustTier,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityContext<'a> {
    pub capability: &'a str,
    pub session_level: u8,
    pub required_level: u8,
    pub declared: &'a HashSet<String>,
    pub entitled: &'a HashSet<String>,
    pub consented: &'a HashSet<String>,
    pub os_granted: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityDecision {
    Allowed,
    NotDeclared,
    NotEntitled,
    ConsentRequired,
    SessionTooWeak,
    OsPermissionRequired,
}

pub fn evaluate_capability(context: &CapabilityContext<'_>) -> CapabilityDecision {
    if !context.declared.contains(context.capability) {
        return CapabilityDecision::NotDeclared;
    }
    if !context.entitled.contains(context.capability) {
        return CapabilityDecision::NotEntitled;
    }
    if context.session_level < context.required_level {
        return CapabilityDecision::SessionTooWeak;
    }
    if !context.consented.contains(context.capability) {
        return CapabilityDecision::ConsentRequired;
    }
    if !context.os_granted {
        return CapabilityDecision::OsPermissionRequired;
    }
    CapabilityDecision::Allowed
}

pub fn verify_artifact(
    bytes: &[u8],
    expected_sha256: &[u8; 32],
    signature: &[u8; 64],
    public_key: &[u8; 32],
) -> bool {
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    digest == *expected_sha256
        && VerifyingKey::from_bytes(public_key)
            .and_then(|key| key.verify(&digest, &Signature::from_bytes(signature)))
            .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn set(values: &[&str]) -> HashSet<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn capability_requires_every_gate() {
        let declared = set(&["message.send"]);
        let entitled = set(&["message.send"]);
        let consented = set(&["message.send"]);
        let mut context = CapabilityContext { capability: "message.send", session_level: 1, required_level: 1, declared: &declared, entitled: &entitled, consented: &consented, os_granted: true };
        assert_eq!(evaluate_capability(&context), CapabilityDecision::Allowed);
        context.session_level = 0;
        assert_eq!(evaluate_capability(&context), CapabilityDecision::SessionTooWeak);
    }

    #[test]
    fn artifact_requires_digest_and_signature() {
        let signing = SigningKey::from_bytes(&[7; 32]);
        let bytes = b"compiled valdi module";
        let digest: [u8; 32] = Sha256::digest(bytes).into();
        let signature = signing.sign(&digest).to_bytes();
        assert!(verify_artifact(bytes, &digest, &signature, &signing.verifying_key().to_bytes()));
        assert!(!verify_artifact(b"tampered", &digest, &signature, &signing.verifying_key().to_bytes()));
    }
}
