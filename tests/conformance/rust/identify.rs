// Copyright 2018 Parity Technologies (UK) Ltd.
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

#![allow(clippy::large_enum_variant)]

use futures::{Stream, StreamExt};
use libp2p::{
    identify, identity, ping,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    PeerId, Swarm,
};
use litep2p::{
    config::Litep2pConfigBuilder,
    crypto::ed25519::Keypair,
    protocol::libp2p::{
        identify::{Config as IdentifyConfig, IdentifyEvent},
        ping::{Config as PingConfig, PingEvent},
    },
    transport::tcp::config::TransportConfig as TcpTransportConfig,
    Litep2p,
};

// We create a custom network behaviour that combines gossipsub, ping and identify.
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "MyBehaviourEvent")]
struct MyBehaviour {
    identify: identify::Behaviour,
    ping: ping::Behaviour,
    // keep_alive: keep_alive::Behaviour,
}

enum MyBehaviourEvent {
    Identify(identify::Event),
    Ping(ping::Event),
}

impl From<identify::Event> for MyBehaviourEvent {
    fn from(event: identify::Event) -> Self {
        MyBehaviourEvent::Identify(event)
    }
}

impl From<ping::Event> for MyBehaviourEvent {
    fn from(event: ping::Event) -> Self {
        MyBehaviourEvent::Ping(event)
    }
}

// initialize litep2p with ping support
async fn initialize_litep2p() -> (
    Litep2p,
    Box<dyn Stream<Item = PingEvent> + Send + Unpin>,
    Box<dyn Stream<Item = IdentifyEvent> + Send + Unpin>,
) {
    let keypair = Keypair::generate();
    let (ping_config, ping_event_stream) = PingConfig::new(3);
    let (identify_config, identify_event_stream) = IdentifyConfig::new();

    let litep2p = Litep2p::new(
        Litep2pConfigBuilder::new()
            .with_keypair(keypair)
            .with_tcp(TcpTransportConfig {
                listen_address: "/ip6/::1/tcp/0".parse().unwrap(),
            })
            .with_ipfs_ping(ping_config)
            .with_ipfs_identify(identify_config)
            .build(),
    )
    .await
    .unwrap();

    (litep2p, ping_event_stream, identify_event_stream)
}

fn initialize_libp2p() -> Swarm<MyBehaviour> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    tracing::debug!("Local peer id: {local_peer_id:?}");

    let transport = libp2p::tokio_development_transport(local_key.clone()).unwrap();
    let behaviour = MyBehaviour {
        identify: identify::Behaviour::new(identify::Config::new(
            "/ipfs/1.0.0".into(),
            local_key.public(),
        )),
        ping: Default::default(),
    };
    let mut swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id).build();

    swarm.listen_on("/ip6/::1/tcp/0".parse().unwrap()).unwrap();

    swarm
}

#[tokio::test]
async fn libp2p_dials() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    let mut libp2p = initialize_libp2p();
    let (mut litep2p, _ping_event_stream, mut identify_event_stream) = initialize_litep2p().await;
    let address = litep2p.listen_addresses().next().unwrap().clone();

    libp2p.dial(address).unwrap();

    tokio::spawn(async move {
        loop {
            let _ = litep2p.next_event().await;
        }
    });

    let mut libp2p_done = false;
    let mut litep2p_done = false;

    loop {
        tokio::select! {
            event = libp2p.select_next_some() => {
                match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        tracing::info!("Listening on {address:?}")
                    }
                    SwarmEvent::Behaviour(MyBehaviourEvent::Ping(_event)) => {},
                    SwarmEvent::Behaviour(MyBehaviourEvent::Identify(_event)) => {
                        libp2p_done = true;

                        if libp2p_done && litep2p_done {
                            break
                        }
                    }
                    _ => {}
                }
            },
            _event = identify_event_stream.next() => {
                litep2p_done = true;

                if libp2p_done && litep2p_done {
                    break
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                panic!("failed to receive identify in time");
            }
        }
    }
}