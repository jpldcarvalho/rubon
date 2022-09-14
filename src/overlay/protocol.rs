
use crate::overlay_proto as proto;

use libp2p_core::upgrade::{InboundUpgrade, OutboundUpgrade, UpgradeInfo};
use libp2p_core::{Multiaddr, PeerId};

use asynchronous_codec::Framed;
use bytes::BytesMut;
use codec::UviBytes;
use futures::prelude::*;
use prost::Message;
use std::borrow::Cow;
use std::{io, iter};
use unsigned_varint::codec;

pub const PROTOCOL_NAME: &[u8] = b"/rubon/overlay/0.1.0";
pub const MAX_PACKET_SIZE: usize = 1024 * 1024 * 4;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestMsg {
    Ping,
    Connect {},
    Disconnect {},
    FindNode {},
    GraftPeer {},
    PrunePeer {},
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

pub struct OverlayProtocolConfig {
    protocol_names: Vec<Cow<'static, [u8]>>,
    max_packet_size: usize,
}

impl OverlayProtocolConfig {
    pub fn protocol_names(&self) -> &[Cow<'static, [u8]>] {
        &self.protocol_names
    }

    pub fn set_protocol_names(&mut self, names: Vec<Cow<'static, [u8]>>) {
        self.protocol_names = names;
    }

    pub fn set_max_packet_size(&mut self, size: usize) {
        self.max_packet_size = size;
    }
}

impl Default for OverlayProtocolConfig {
    fn default() -> Self {
        OverlayProtocolConfig {
            protocol_names: vec![Cow::Borrowed(PROTOCOL_NAME)],
            max_packet_size: MAX_PACKET_SIZE
        }
    }
}

pub type OverlayStreamSink<S, A, B> = stream::AndThen<
    sink::With<
        stream::ErrInto<Framed<S, UviBytes<io::Cursor<Vec<u8>>>>, io::Error>,
        io::Cursor<Vec<u8>>,
        A,
        future::Ready<Result<io::Cursor<Vec<u8>>, io::Error>>,
        fn(A) -> future::Ready<Result<io::Cursor<Vec<u8>>, io::Error>>,
    >,
    future::Ready<Result<B, io::Error>>,
    fn(BytesMut) -> future::Ready<Result<B, io::Error>>,
>;

pub type OverlayInStreamSink<S> = OverlayStreamSink<S, ResponseMsg, RequestMsg>;
pub type OverlayOutStreamSink<S> = OverlayStreamSink<S, RequestMsg, ResponseMsg>;

impl UpgradeInfo for OverlayProtocolConfig {
    type Info = Cow<'static, [u8]>;
    type InfoIter = std::vec::IntoIter<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        self.protocol_names.clone().into_iter()
    }
}

impl<C> InboundUpgrade<C> for OverlayProtocolConfig
where C: AsyncRead + AsyncWrite + Unpin
{
    type Output = OverlayInStreamSink<C>;
    type Future = future::Ready<Result<Self::Output, io::Error>>;
    type Error = io::Error;

    fn upgrade_inbound(self, incoming: C, _: Self::Info) -> Self::Future {
        let mut codec = UviBytes::default();
        codec.set_max_len(self.max_packet_size);

        future::ok(
            Framed::new(incoming, codec)
                .err_into()
                .with::<_, _, fn(_) -> _, _>(|resp| {
                    let proto_struct = resp_msg_to_proto(resp);
                    let mut buf = Vec::with_capacity(proto_struct.encoded_len());
                    proto_struct
                        .encode(&mut buf)
                        .expect("Vec<u8> is a valid protobuf message");
                    future::ready(Ok(io::Cursor::new(buf)))
                
                })
                .and_then::<_, fn() -> _>(|bytes| {
                    let req = match proto::Message::decode(bytes) {
                        Ok(req) => req,
                        Err(err) => return future::ready(Err(err.into()))
                    };
                    future::ready(proto_to_req_msg(req))
                }),
        )
    }
}

impl<C> OutboundUpgrade<C> for OverlayProtocolConfig
where C: AsyncRead + AsyncWrite + Unpin
{
    type Output = OverlayOutStreamSink<C>;
    type Future = future::Ready<Result<Self::Output, io::Error>>;
    type Error = io::Error;

    fn upgrade_outbound(self, incoming: C, _: Self::Info) -> Self::Future {
        let mut codec = UviBytes::default();
        codec.set_max_len(self.max_packet_size);

        future::ok(
            Framed::new(incoming, codec)
                .err_into()
                .with::<_, _, fn(_) -> _, _>(|resp| {
                    let proto_struct = req_msg_to_proto(resp);
                    let mut buf = Vec::with_capacity(proto_struct.encoded_len());
                    proto_struct
                        .encode(&mut buf)
                        .expect("Vec<u8> is a valid protobuf message");
                    future::ready(Ok(io::Cursor::new(buf)))
                })
                .and_then::<_, fn() -> _>(|bytes| {
                    let resp = match proto::Message::decode(bytes) {
                        Ok(req) => req,
                        Err(err) => return future::ready(Err(err.into()))
                    };
                    future::ready(proto_to_resp_msg(resp))
                }),
        )
    }
}

