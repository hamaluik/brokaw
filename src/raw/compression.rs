use async_compression::futures::bufread::ZlibDecoder;
use async_std::io;
use async_std::io::{BufRead, BufReader, Read};
use std::marker::Unpin;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A type of compression enabled on the server
#[derive(Copy, Clone, Debug)]
pub enum Compression {
    /// Giganews style compression
    XFeature,
}

/// An codec that can unpack compressed data streams
#[derive(Debug)]
pub(crate) enum Decoder<S: BufRead + Unpin> {
    XFeature(BufReader<ZlibDecoder<S>>),
    Passthrough(S),
}

impl Compression {
    pub(crate) fn use_decoder(&self, first_line: impl AsRef<[u8]>) -> bool {
        match self {
            Self::XFeature => first_line.as_ref().ends_with(b"[COMPRESS=GZIP]\r\n"),
        }
    }

    pub(crate) fn decoder<S: BufRead + Read + Unpin>(&self, stream: S) -> Decoder<S> {
        match self {
            Self::XFeature => Decoder::XFeature(BufReader::new(ZlibDecoder::new(stream))),
        }
    }
}

impl<S: Read + BufRead + Unpin> Read for Decoder<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        match self.get_mut() {
            Decoder::XFeature(d) => Pin::new(d).poll_read(cx, buf),
            Decoder::Passthrough(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl<S: BufRead + Unpin> BufRead for Decoder<S> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        match self.get_mut() {
            Decoder::XFeature(d) => Pin::new(d).poll_fill_buf(cx),
            Decoder::Passthrough(s) => Pin::new(s).poll_fill_buf(cx),
        }
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        match self.get_mut() {
            Decoder::XFeature(d) => Pin::new(d).consume(amt),
            Decoder::Passthrough(s) => Pin::new(s).consume(amt),
        }
    }
}

/*
    In theory if we wanted to implement extensible compression we could replace Decoder and
    Compression objects w/ traits. That said it didn't seem necessary given the slow moving
    nature of the NNTP standard. If users ask for this we can always revisit it.
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use_decoder() {
        assert!(
            Compression::XFeature.use_decoder("224 xover information follows [COMPRESS=GZIP]\r\n")
        );
        assert!(!Compression::XFeature.use_decoder("224 xover information follows [COMPRESS=GZIP]"))
    }

    #[test]
    fn test_compressed() {
        let compressed_resp = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/xover_resp_xfeature_compress"
        ));
        let plain_resp = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/xover_resp_plain_text"
        ));

        let line_boundary = compressed_resp
            .iter()
            .enumerate()
            .find(|(_i, &byte)| byte == b'\n')
            .map(|(i, _)| i)
            .unwrap();

        let (first_line, data_blocks) = (
            &compressed_resp[..line_boundary + 1],
            &compressed_resp[line_boundary + 1..],
        );

        assert!(Compression::XFeature.use_decoder(first_line));

        let mut decoder = Compression::XFeature.decoder(&data_blocks[..]);
        let mut buf = String::new();
        // TODO: async testing
        //decoder.read_to_string(&mut buf).unwrap();
        //assert_eq!(buf, String::from_utf8(plain_resp.to_vec()).unwrap())
    }
}
