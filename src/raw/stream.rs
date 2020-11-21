use async_std::net::TcpStream;

use async_std::io;
use async_std::io::{Read, Write};
use async_tls::client::TlsStream;

use std::pin::Pin;
use std::task::{Context, Poll};

/// A raw NNTP session
#[derive(Debug)]
pub enum NntpStream {
    /// A stream using TLS
    Tls(TlsStream<TcpStream>),
    /// A plain text stream
    Tcp(TcpStream),
}

impl From<TlsStream<TcpStream>> for NntpStream {
    fn from(stream: TlsStream<TcpStream>) -> Self {
        Self::Tls(stream)
    }
}

impl From<TcpStream> for NntpStream {
    fn from(stream: TcpStream) -> NntpStream {
        Self::Tcp(stream)
    }
}

impl Read for NntpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        match self.get_mut() {
            NntpStream::Tls(stream) => Pin::new(stream).poll_read(cx, buf),
            NntpStream::Tcp(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl Write for NntpStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.get_mut() {
            NntpStream::Tls(stream) => Pin::new(stream).poll_write(cx, buf),
            NntpStream::Tcp(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            NntpStream::Tls(stream) => Pin::new(stream).poll_flush(cx),
            NntpStream::Tcp(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            NntpStream::Tls(stream) => Pin::new(stream).poll_close(cx),
            NntpStream::Tcp(stream) => Pin::new(stream).poll_close(cx),
        }
    }
}
