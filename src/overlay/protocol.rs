
use crate::overlay_proto as proto;

use libp2p_core::{Multiaddr, PeerId};

use std::io;

pub const PROTOCOL_NAME: &[u8] = b"/ziku/overlay/0.1.0";

pub enum ConnectionType {
    NotConnected = 0,
    Connected = 1,
    CanConnect = 2,
    CannotConnect = 3
}

impl From<proto::message::ConnectionType> for ConnectionType {
    fn from(raw: proto::message::ConnectionType) -> ConnectionType {
        use proto::message::ConnectionType::*;
        match raw {
            NotConnected => ConnectionType::NotConnected,
            Connected => ConnectionType::Connected,
            CanConnect => ConnectionType::CanConnect,
            CannotConnect => ConnectionType::CannotConnect
        }
    }
}

impl From<ConnectionType> for proto::message::ConnectionType {
    fn from(val: ConnectionType) -> Self {
        use proto::message::ConnectionType::*;
        match val {
            ConnectionType::NotConnected => NotConnected,
            ConnectionType::Connected => Connected,
            ConnectionType::CanConnect => CanConnect,
            ConnectionType::CannotConnect => CannotConnect
        }
    }
}

pub struct Peer {
    pub id: PeerId,
    pub multiaddrs: Vec<Multiaddr>,
    pub conn_type: ConnectionType
}

impl From<Peer> for proto::message::Peer {
    fn from(peer: Peer) -> Self {
        proto::message::Peer {
            id: peer.id.to_bytes(),
            addrs: peer.multiaddrs.into_iter().map(|a| a.to_vec()).collect(),
            connection: {
                let conn_type: proto::message::ConnectionType = peer.conn_type.into();
                conn_type as i32
            }
        }
    }
}

pub enum RequestMsg {
    Ping,
    Connect {},
    Disconnect {},
    FindNode {},
    GraftPeer {},
    PrunePeer {},
}

pub enum ResponseMsg {
    Pong,
    ConnectAck {},
    Neighbors {}
}

fn req_msg_to_proto(msg: RequestMsg) -> proto::Message {
    match msg {
        RequestMsg::Ping => proto::Message {
            r#type: proto::Message::Ping as i32,
            ..proto::Message::default()
        },
        RequestMsg::Connect {} => proto::Message {
            r#type: proto::Message::Connect as i32,
            ..proto::Message::default()
        },
        RequestMsg::Disconnect {} => proto::Message {
            r#type: proto::Message::Disconnect as i32,
            ..proto::Message::default()
        },
        RequestMsg::FindNode {} => proto::Message {
            r#type: proto::Message::FindNode as i32,
            ..proto::Message::default()
        },
        RequestMsg::GraftPeer {} => proto::Message {
            r#type: proto::Message::GraftPeer as i32,
            ..proto::Message::default()
        },
        RequestMsg::PrunePeer {} => proto::Message {
            r#type: proto::Message::PrunePeer as i32,
            ..proto::Message::default()
        }
    }
}

fn resp_msg_to_proto(msg: ResponseMsg) -> proto::Message {}

fn proto_to_req_msg(msg: proto::Message) -> Result<RequestMsg, io::Error> {}




