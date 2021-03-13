use bytes::{BufMut, BytesMut};
use matchbook_types::*;
use std::net::SocketAddrV4;
use tokio_util::codec::{Decoder, Encoder};

pub struct MatchbookCodec {
    buf: Vec<u8>,
}

impl MatchbookCodec {
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(1024),
        }
    }
}

impl Encoder<Message> for MatchbookCodec {
    type Error = Box<dyn std::error::Error>;

    fn encode(
        &mut self,
        item: Message,
        dst: &mut BytesMut,
    ) -> Result<(), <Self as Encoder<Message>>::Error> {
        serde_json::to_writer(&mut self.buf, &item)?;
        dst.put(&self.buf[..self.buf.len()]);
        self.buf.clear();
        Ok(())
    }
}

impl Decoder for MatchbookCodec {
    type Item = Message;

    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            Ok(None)
        } else {
            let maybe_message = serde_json::from_slice(src).map(Some).map_err(|e| e.into());
            src.clear();
            maybe_message
        }
    }
}

/// Bind socket to multicast address with IP_MULTICAST_LOOP and SO_REUSEADDR Enabled
pub fn bind_multicast(
    addr: &SocketAddrV4,
    multi_addr: &SocketAddrV4,
) -> Result<std::net::UdpSocket, Box<dyn std::error::Error>> {
    use socket2::{Domain, Protocol, Socket, Type};

    assert!(multi_addr.ip().is_multicast(), "Must be multcast address");

    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;

    socket.set_reuse_address(true)?;
    socket.bind(&socket2::SockAddr::from(*addr))?;
    socket.set_nonblocking(true)?;
    socket.set_multicast_loop_v4(false)?;
    socket.join_multicast_v4(multi_addr.ip(), addr.ip())?;

    Ok(socket.into_udp_socket())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
