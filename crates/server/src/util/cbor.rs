use std::marker::PhantomData;

use bytes::{Buf, BytesMut};
use tokio_util::codec::Decoder;

pub struct CborDecoder<T>(PhantomData<T>);

impl<T> Default for CborDecoder<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<'de, T: serde::Deserialize<'de>> Decoder for CborDecoder<T> {
    type Item = T;
    type Error = ciborium::de::Error<std::io::Error>;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut bytes: &[u8] = src.as_ref();
        let start = bytes.len();

        let item: T = match ciborium::de::from_reader(&mut bytes) {
            // Using a wildcard match here because decoding from a byte slice can only produce a
            // `ciborium_io::EndOfFile` struct, which has a private constructor.
            Err(ciborium::de::Error::Io(_)) => return Ok(None),
            Ok(v) => v,
            err => err?,
        };

        let end = bytes.len();
        src.advance(start - end);

        Ok(Some(item))
    }
}
