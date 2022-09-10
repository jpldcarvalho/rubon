
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
            r#type: proto::message::MessageType::Ping as i32,
            ..proto::Message::default()
        },
        RequestMsg::Connect {} => proto::Message {
            r#type: proto::message::MessageType::Connect as i32,
            ..proto::Message::default()
        },
        RequestMsg::Disconnect {} => proto::Message {
            r#type: proto::message::MessageType::Disconnect as i32,
            ..proto::Message::default()
        },
        RequestMsg::FindNode {} => proto::Message {
            r#type: proto::message::MessageType::FindNode as i32,
            ..proto::Message::default()
        },
        RequestMsg::GraftPeer {} => proto::Message {
            r#type: proto::message::MessageType::GraftPeer as i32,
            ..proto::Message::default()
        },
        RequestMsg::PrunePeer {} => proto::Message {
            r#type: proto::message::MessageType::PrunePeer as i32,
            ..proto::Message::default()
        }
    }
}

fn resp_msg_to_proto(msg: ResponseMsg) -> proto::Message {
    match msg {
        ResponseMsg::Pong => proto::Message {
            r#type: proto::message::MessageType::Ping as i32,
            ..proto::Message::default()
        },
        ResponseMsg::ConnectAck {} => proto::Message {
            r#type: proto::message::MessageType::Connect as i32,
            ..proto::Message::default()
        },
        ResponseMsg::Neighbors {} => proto::Message {
            r#type: proto::message::MessageType::FindNode as i32,
            ..proto::Message::default()
        }
    }
}

fn proto_to_req_msg(msg: proto::Message) -> Result<RequestMsg, io::Error> {
    let msg_type = proto::message::MessageType::from_i32(msg.r#type)
        .ok_or_else(|| {io::Error::new(io::ErrorKind::InvalidData, "Invalid message type")})?;
    match msg_type {
        proto::message::MessageType::Ping => Ok(RequestMsg::Ping),
        proto::message::MessageType::Connect => Ok(RequestMsg::Connect {}),
        proto::message::MessageType::Disconnect => Ok(RequestMsg::Disconnect {}),
        proto::message::MessageType::FindNode => Ok(RequestMsg::FindNode {}),
        proto::message::MessageType::GraftPeer => Ok(RequestMsg::GraftPeer {}),
        proto::message::MessageType::PrunePeer => Ok(RequestMsg::PrunePeer {}),
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid message type"))
    }
}

fn proto_to_resp_msg(msg: proto::Message) -> Result<ResponseMsg, io::Error> {
    let msg_type = proto::message::MessageType::from_i32(msg.r#type)
        .ok_or_else(|| {io::Error::new(io::ErrorKind::InvalidData, "Invalid message type")})?;
    match msg_type {
        proto::message::MessageType::Ping => Ok(ResponseMsg::Pong),
        proto::message::MessageType::Connect => Ok(ResponseMsg::ConnectAck {}),
        proto::message::MessageType::FindNode => Ok(ResponseMsg::Neighbors {}),
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid message type"))
    }
}

