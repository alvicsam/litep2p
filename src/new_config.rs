// Copyright 2023 litep2p developers
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use crate::{
    crypto::ed25519::Keypair,
    protocol::{notification_new, request_response_new, Protocol},
    transport::tcp_new::config,
    types::protocol::ProtocolName,
};

use multiaddr::Multiaddr;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Litep2pConfigBuilder {
    // TCP transport configuration.
    tcp: Option<config::TransportConfig>,

    /// Keypair.
    keypair: Option<Keypair>,

    /// Notification protocols.
    notification_protocols: HashMap<ProtocolName, notification_new::types::Config>,

    /// Request-response protocols.
    request_response_protocols: HashMap<ProtocolName, request_response_new::types::Config>,
}

impl Litep2pConfigBuilder {
    /// Create new empty [`LiteP2pConfigBuilder`].
    pub fn new() -> Self {
        Self {
            tcp: None,
            keypair: None,
            notification_protocols: HashMap::new(),
            request_response_protocols: HashMap::new(),
        }
    }

    /// Add TCP transport configuration.
    pub fn with_tcp(mut self, config: config::TransportConfig) -> Self {
        self.tcp = Some(config);
        self
    }

    /// Add keypair.
    pub fn with_keypair(mut self, keypair: Keypair) -> Self {
        self.keypair = Some(keypair);
        self
    }

    /// Install notification protocol.
    pub fn with_notification_protocol(mut self, config: notification_new::types::Config) -> Self {
        self.notification_protocols
            .insert(config.protocol_name().clone(), config);
        self
    }

    /// Install request-response protocol.
    pub fn with_request_response_protocol(
        mut self,
        config: request_response_new::types::Config,
    ) -> Self {
        self.request_response_protocols
            .insert(config.protocol_name().clone(), config);
        self
    }

    /// Build [`Litep2pConfig`].
    ///
    /// Generates a default keypair if user didn't provide one.
    pub fn build(mut self) -> Litep2pConfig {
        let keypair = match self.keypair {
            Some(keypair) => keypair,
            None => Keypair::generate(),
        };

        Litep2pConfig {
            keypair,
            tcp: self.tcp.take(),
            notification_protocols: self.notification_protocols,
            request_response_protocols: self.request_response_protocols,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    // TCP transport configuration.
    tcp: Option<config::TransportConfig>,

    /// Keypair.
    keypair: Keypair,

    /// Enabled protocols.
    protocols: Vec<ProtocolName>,
}

// TODO: remove this and introduce some context object instead
impl Config {
    /// Get keypair.
    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    /// Get TCP transport configuration.
    pub fn tcp(&self) -> &Option<config::TransportConfig> {
        &self.tcp
    }

    /// Get a list of enabled protocols.
    pub fn protocols(&self) -> &Vec<ProtocolName> {
        &self.protocols
    }
}

impl From<&Litep2pConfig> for Config {
    fn from(config: &Litep2pConfig) -> Self {
        let protocols = {
            let mut protocols = Vec::new();

            config
                .notification_protocols
                .iter()
                .for_each(|(name, _)| protocols.push(name.clone()));
            config
                .request_response_protocols
                .iter()
                .for_each(|(name, _)| protocols.push(name.clone()));

            protocols
        };

        Self {
            keypair: config.keypair.clone(),
            tcp: config.tcp.clone(),
            protocols,
        }
    }
}

#[derive(Debug)]
pub struct Litep2pConfig {
    // TCP transport configuration.
    tcp: Option<config::TransportConfig>,

    /// Keypair.
    keypair: Keypair,

    /// Notification protocols.
    notification_protocols: HashMap<ProtocolName, notification_new::types::Config>,

    /// Request-response protocols.
    request_response_protocols: HashMap<ProtocolName, request_response_new::types::Config>,
}

impl Litep2pConfig {
    /// Get keypair.
    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    /// Get TCP transport configuration.
    pub fn tcp(&self) -> &Option<config::TransportConfig> {
        &self.tcp
    }
}
