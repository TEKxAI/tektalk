//! Transport selection policy shared by all native TEKtalk clients.
//!
//! Actual socket adapters are platform/runtime integrations. This module owns
//! the deterministic TCP-first and WSS-fallback decision so every client
//! applies the same security and retry rules.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportKind {
    NativeTcp,
    SecureWebSocket,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureClass {
    /// DNS, connection refusal, timeout, proxy rejection or network loss.
    Unavailable,
    /// Authentication, server identity, fingerprint or message-key failure.
    Security,
    /// A peer violated MTProto framing or session invariants.
    Protocol,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FallbackDecision {
    Connect(TransportKind),
    Stop,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndpointSet {
    pub tcp: String,
    pub wss: String,
}

#[derive(Debug, Default)]
pub struct TransportManager {
    tcp_attempted: bool,
    wss_attempted: bool,
    tcp_blocked_for_network: bool,
}

impl TransportManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts with native TCP unless this network has recently rejected it.
    pub fn begin(&mut self) -> FallbackDecision {
        if !self.tcp_blocked_for_network {
            self.tcp_attempted = true;
            FallbackDecision::Connect(TransportKind::NativeTcp)
        } else {
            self.wss_attempted = true;
            FallbackDecision::Connect(TransportKind::SecureWebSocket)
        }
    }

    /// Only network availability failures may fall back. Security and protocol
    /// failures stop immediately so a hostile endpoint cannot downgrade the
    /// connection path.
    pub fn failed(
        &mut self,
        transport: TransportKind,
        failure: FailureClass,
    ) -> FallbackDecision {
        if failure != FailureClass::Unavailable {
            return FallbackDecision::Stop;
        }
        match transport {
            TransportKind::NativeTcp if !self.wss_attempted => {
                self.tcp_blocked_for_network = true;
                self.wss_attempted = true;
                FallbackDecision::Connect(TransportKind::SecureWebSocket)
            }
            _ => FallbackDecision::Stop,
        }
    }

    /// A network change invalidates the cached TCP-blocked observation.
    pub fn network_changed(&mut self) {
        self.tcp_attempted = false;
        self.wss_attempted = false;
        self.tcp_blocked_for_network = false;
    }

    pub fn tcp_was_attempted(&self) -> bool {
        self.tcp_attempted
    }

    pub fn is_using_cached_wss_fallback(&self) -> bool {
        self.tcp_blocked_for_network
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_tcp_then_wss_for_network_failure() {
        let mut manager = TransportManager::new();
        assert_eq!(
            manager.begin(),
            FallbackDecision::Connect(TransportKind::NativeTcp)
        );
        assert_eq!(
            manager.failed(TransportKind::NativeTcp, FailureClass::Unavailable),
            FallbackDecision::Connect(TransportKind::SecureWebSocket)
        );
        assert!(manager.is_using_cached_wss_fallback());
    }

    #[test]
    fn never_downgrades_after_security_or_protocol_failure() {
        for failure in [FailureClass::Security, FailureClass::Protocol] {
            let mut manager = TransportManager::new();
            manager.begin();
            assert_eq!(
                manager.failed(TransportKind::NativeTcp, failure),
                FallbackDecision::Stop
            );
        }
    }

    #[test]
    fn cached_block_uses_wss_until_network_changes() {
        let mut manager = TransportManager::new();
        manager.begin();
        manager.failed(TransportKind::NativeTcp, FailureClass::Unavailable);
        assert_eq!(
            manager.begin(),
            FallbackDecision::Connect(TransportKind::SecureWebSocket)
        );
        manager.network_changed();
        assert_eq!(
            manager.begin(),
            FallbackDecision::Connect(TransportKind::NativeTcp)
        );
        assert!(manager.tcp_was_attempted());
    }

    #[test]
    fn does_not_loop_when_wss_is_unavailable() {
        let mut manager = TransportManager::new();
        manager.begin();
        manager.failed(TransportKind::NativeTcp, FailureClass::Unavailable);
        assert_eq!(
            manager.failed(
                TransportKind::SecureWebSocket,
                FailureClass::Unavailable
            ),
            FallbackDecision::Stop
        );
    }
}
