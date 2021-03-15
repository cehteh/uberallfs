use core::mem::{self, MaybeUninit};
use std::fmt::{self, Debug};
use base64;
use std::io;

use crate::identifier_kind::*;
use crate::rev_cursor;

// config vars in case this ever changes
const BITS_IN_BINARY_ID: usize = 256;
const KIND_ID_LEN: usize = 1;

/**
Identifiers are generated from a 'IdentifierKind' descriptor and a 'IdentifierBin' hash or
random number. They represented as 'flipbase64' strings. That is base64 encoded strings of
size FLIPBASE64_LEN written backwards. This backwards encoding allows fair distribution within
the objectstore directory hierarchy while still encoding the 'IdentifierKind' first and only
decode the first (last) 2 bytes for its decoding (the decoded identifier itself is rarely
needed).
**/

const BINARY_ID_LEN: usize = (BITS_IN_BINARY_ID + 7) / 8;
const FLIPBASE64_LEN: usize = (BITS_IN_BINARY_ID + KIND_ID_LEN*8 + 5) / 6;
const BASE64_AGGREGATE: usize = 4; // for valid non padded en/decoding base64 length must be a multiple of this

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct IdentifierBin(pub [u8; BINARY_ID_LEN]);

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Flipbase64(pub [u8; FLIPBASE64_LEN]);

#[derive(Debug, PartialEq)]
pub(crate) struct Identifier {
    kind: IdentifierKind,
    base64: Flipbase64,
}


impl From<&Flipbase64> for IdentifierBin {
    fn from(base64: &Flipbase64) -> Self {
        use std::convert::TryInto;
        use std::io::Read;

        let mut cursor = rev_cursor::ReadCursor::from(&base64.0[..]);
        let mut decoder = base64::read::DecoderReader::new(&mut cursor, base64::URL_SAFE_NO_PAD);

        let mut blob = [0u8; BINARY_ID_LEN + KIND_ID_LEN];
        decoder.read(&mut blob).unwrap();

        IdentifierBin(blob[1..].try_into().unwrap())
    }
}

impl From<&Flipbase64> for IdentifierKind {
    fn from(base64: &Flipbase64) -> Self {
        use std::io::Read;

        let mut cursor = rev_cursor::ReadCursor::from(&base64.0[FLIPBASE64_LEN-BASE64_AGGREGATE..]);
        let mut decoder = base64::read::DecoderReader::new(&mut cursor, base64::URL_SAFE_NO_PAD);

        let mut kind = [0u8; 1];
        decoder.read(&mut kind).unwrap();

        IdentifierKind(kind[0])
    }
}


impl Identifier {
    pub(crate) fn from_binary(kind: IdentifierKind, binary: IdentifierBin) -> Identifier {
        use std::io::Write;

        let mut base64: [MaybeUninit<u8>; FLIPBASE64_LEN] = MaybeUninit::uninit_array();
        let mut encoder = base64::write::EncoderWriter::new(
            rev_cursor::WriteCursor::new(&mut base64[..]),
            base64::URL_SAFE_NO_PAD,
        );

        encoder.write(&[kind.0]).unwrap();
        encoder.write(&binary.0).unwrap();
        drop(encoder);

        Identifier {
            kind,
            base64: unsafe { Flipbase64(MaybeUninit::array_assume_init(base64)) },
        }
    }

    pub(crate) fn from_flipbase64(base64: Flipbase64) -> Identifier {
        Identifier {
            kind: (&base64).into(),
            base64,
        }
    }

    pub(crate) fn id_base64(&self) -> &Flipbase64 {
        &self.base64
    }

    pub(crate) fn id_bin(&self) -> IdentifierBin {
        (&self.base64).into()
    }

    pub(crate) fn object_type(&self) -> ObjectType {
        self.kind.object_type()
    }

    pub(crate) fn sharing_policy(&self) -> SharingPolicy {
        self.kind.sharing_policy()
    }

    pub(crate) fn mutability(&self) -> Mutability {
        self.kind.mutability()
    }
}

