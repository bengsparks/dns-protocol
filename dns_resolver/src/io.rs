use std::{
    net::{self, SocketAddr},
    task::ready,
};

use tokio::net::{TcpStream, UdpSocket};

pub struct Config {
    udp: SocketAddr,
    tcp: SocketAddr,
}

pub struct DnsSockets {
    udp: UdpSocket,
    tcp: TcpStream,
}

impl DnsSockets {
    pub async fn bind(config: Config) -> Result<Self, std::io::Error> {
        DnsSockets {
            udp: UdpSocket::bind(config.udp).await?,
            tcp: TcpStream::connect(config.tcp).await?,
        }
    }
}

impl tokio::io::AsyncRead for DnsSockets {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let (read, _) = self.tcp.split();

        let status = [self.udp.poll_recv(cx, buf), read.poll_read(cx, buf)]
            .into_iter()
            .find(|poll| poll.is_ready());

        status.unwrap_or(std::task::Poll::Pending)
    }
}

impl tokio::io::AsyncWrite for DnsSockets {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        if buf.len() <= 512 {
            self.udp.poll_send(cx, buf)
        } else {
            let (_, write) = self.tcp.split();
            write.poll_write(cx, buf)
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        todo!()
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}
