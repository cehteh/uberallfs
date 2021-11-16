use std::error::Error;
use std::ffi::OsStr;
use std::hash::Hash;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::path::PathBuf;
use core::mem::MaybeUninit;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt::{self, Debug};

use arrayref::array_ref;
use unchecked_unwrap::UncheckedUnwrap;
use uberall::cachedb::*;

use crate::objectpath::ObjectPath;
use crate::prelude::*;
use crate::identifier_kind::*;
use crate::rev_cursor;

// config vars in case this ever changes
const BITS_IN_BINARY_ID: usize = 256;
const KIND_ID_LEN: usize = 1;

/// Identifiers are generated from a 'IdentifierKind' descriptor and a
/// 'IdentifierBin' hash or random number. They represented as 'flipbase64'
/// strings. That is base64 encoded strings of size FLIPBASE64_LEN written
/// backwards. This backwards encoding allows fair distribution within
/// the objectstore directory hierarchy while still encoding the
/// 'IdentifierKind' first and only decode the first (last) 2 bytes for its
/// decoding (the decoded identifier itself is rarely needed).

const BINARY_ID_LEN: usize = (BITS_IN_BINARY_ID + 7) / 8;
const FLIPBASE64_LEN: usize = (BITS_IN_BINARY_ID + KIND_ID_LEN * 8 + 5) / 6;
const BASE64_AGGREGATE: usize = 4; // for valid non padded en/decoding base64 length must be a multiple of this

#[derive(Debug, PartialEq, Clone)]
pub struct IdentifierBin(pub [u8; BINARY_ID_LEN]);

#[derive(Debug, PartialEq, Clone)]
pub struct Flipbase64(pub [u8; FLIPBASE64_LEN]);

#[derive(Clone)]
pub struct Identifier {
    kind:   IdentifierKind,
    base64: Flipbase64,
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.base64 == other.base64
    }
}

impl Hash for Identifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.base64.0.hash(state)
    }
}

impl Eq for Identifier {}

impl Bucketize for Identifier {
    fn bucket<const N: usize>(&self) -> usize {
        self.base64.0[0..4]
            .iter()
            .map(|x| *x as usize)
            .sum::<usize>()
            % N
    }
}

impl KeyTraits for Identifier {}

pub struct IdentifierBuilder(IdentifierKind);

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("Identifier")
            .field("kind", &self.kind.components())
            .field("base64", &OsStr::from_bytes(&self.base64.0[..]))
            .finish()
    }
}

impl TryFrom<&Flipbase64> for IdentifierBin {
    type Error = Box<dyn Error>;

    fn try_from(base64: &Flipbase64) -> Result<Self> {
        use io::Read;

        let mut cursor = rev_cursor::ReadCursor::from(&base64.0[..]);
        let mut decoder = base64::read::DecoderReader::new(&mut cursor, base64::URL_SAFE_NO_PAD);

        let mut buffer = [0u8; BINARY_ID_LEN + KIND_ID_LEN];
        decoder.read_exact(&mut buffer)?;
        let id = unsafe { buffer[1..].try_into().unchecked_unwrap() };

        Ok(IdentifierBin(id))
    }
}

impl TryFrom<&Flipbase64> for IdentifierKind {
    type Error = Box<dyn Error>;

    fn try_from(base64: &Flipbase64) -> Result<Self> {
        use io::Read;

        let mut cursor =
            rev_cursor::ReadCursor::from(&base64.0[FLIPBASE64_LEN - BASE64_AGGREGATE..]);
        let mut decoder = base64::read::DecoderReader::new(&mut cursor, base64::URL_SAFE_NO_PAD);
        let mut kind = [0u8; 1];
        decoder.read_exact(&mut kind)?;

        Ok(IdentifierKind(kind[0]))
    }
}

impl Identifier {
    pub fn ensure_dir(&self) -> Result<&Self> {
        if self.object_type() == ObjectType::Directory {
            Ok(self)
        } else {
            Err(ObjectStoreError::ObjectType {
                have: self.object_type(),
                want: ObjectType::Directory,
            }
            .into())
        }
    }

    pub fn ensure_file(&self) -> Result<&Self> {
        if self.object_type() == ObjectType::File {
            Ok(self)
        } else {
            Err(ObjectStoreError::ObjectType {
                have: self.object_type(),
                want: ObjectType::File,
            }
            .into())
        }
    }

    pub(crate) fn from_binary(kind: IdentifierKind, binary: IdentifierBin) -> Identifier {
        use io::Write;

        let mut base64: [MaybeUninit<u8>; FLIPBASE64_LEN] = MaybeUninit::uninit_array();
        let mut encoder = base64::write::EncoderWriter::new(
            rev_cursor::WriteCursor::new(&mut base64[..]),
            base64::URL_SAFE_NO_PAD,
        );

        unsafe {
            encoder.write(&[kind.0]).unchecked_unwrap();
            encoder.write(&binary.0).unchecked_unwrap();
        }
        drop(encoder);

        Identifier {
            kind,
            base64: unsafe { Flipbase64(MaybeUninit::array_assume_init(base64)) },
        }
    }

    pub(crate) fn build(kind: IdentifierKind) -> IdentifierBuilder {
        IdentifierBuilder(kind)
    }

    pub(crate) fn from_flipbase64(base64: Flipbase64) -> Result<Identifier> {
        Ok(Identifier {
            kind: (&base64).try_into()?,
            base64,
        })
    }

    pub(crate) fn from_filename(path: &Path) -> Result<Identifier> {
        let filename = path.file_name().ok_or(ObjectStoreError::InvalidIdentifier(
            path.to_string_lossy().to_string(),
        ))?;

        if filename.len() == 44 {
            Identifier::from_flipbase64(Flipbase64(*array_ref![filename.as_bytes(), 0, 44]))
        } else {
            Err(ObjectStoreError::InvalidIdentifier(filename.to_string_lossy().to_string()).into())
        }
    }

    pub(crate) fn id_base64(&self) -> &Flipbase64 {
        &self.base64
    }

    pub(crate) fn as_os_str(&self) -> &OsStr {
        OsStr::from_bytes(&self.base64.0[..])
    }

    pub(crate) fn id_bin(&self) -> IdentifierBin {
        unsafe { (&self.base64).try_into().unchecked_unwrap() }
    }

    pub(crate) fn kind(&self) -> IdentifierKind {
        self.kind
    }

    pub fn object_type(&self) -> ObjectType {
        self.kind.object_type()
    }

    pub fn sharing_policy(&self) -> SharingPolicy {
        self.kind.sharing_policy()
    }

    pub fn mutability(&self) -> Mutability {
        self.kind.mutability()
    }

    pub fn components(&self) -> (ObjectType, SharingPolicy, Mutability) {
        (
            self.kind.object_type(),
            self.kind.sharing_policy(),
            self.kind.mutability(),
        )
    }

    /// create a objectstore path from an identifier
    #[cfg(unix)]
    pub fn to_pathbuf(&self) -> PathBuf {
        let mut pathbuf = PathBuf::new();
        pathbuf.push_identifier(self);
        pathbuf
    }
}

impl IdentifierBuilder {
    pub(crate) fn with_binary(self, binary: IdentifierBin) -> Identifier {
        use io::Write;

        let mut base64: [MaybeUninit<u8>; FLIPBASE64_LEN] = MaybeUninit::uninit_array();
        let mut encoder = base64::write::EncoderWriter::new(
            rev_cursor::WriteCursor::new(&mut base64[..]),
            base64::URL_SAFE_NO_PAD,
        );

        unsafe {
            encoder.write(&[self.0.0]).unchecked_unwrap();
            encoder.write(&binary.0).unchecked_unwrap();
        }
        drop(encoder);

        Identifier {
            kind:   self.0,
            base64: unsafe { Flipbase64(MaybeUninit::array_assume_init(base64)) },
        }
    }

    pub(crate) fn components(&self) -> (ObjectType, SharingPolicy, Mutability) {
        (
            self.0.object_type(),
            self.0.sharing_policy(),
            self.0.mutability(),
        )
    }
}
