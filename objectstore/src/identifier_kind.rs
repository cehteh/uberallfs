// Define every possible value, this allows for unsafe conversions from bytes values to enums
// with transmute without checking because this is exhaustive and infallible. Errors will be
// handled when matching. Reserved entries starting with an underscore and are ideas w/o any
// implementation yet.

#[repr(u8)] // 3 bits
#[derive(Debug, PartialEq)]
pub enum ObjectType {
    File = 0 << 5,
    Directory = 1 << 5,
    _PartialFile = 2 << 5,
    _FecBlock = 3 << 5,
    _DirectoryWithParent = 4 << 5,
    _Reserved3 = 5 << 5,
    _Reserved4 = 6 << 5,
    _Reserved5 = 7 << 5,
}

#[repr(u8)] // 3 bits
#[derive(Debug, PartialEq)]
pub enum SharingPolicy {
    Private = 0 << 2,
    PublicAcl = 1 << 2,
    Anonymous = 2 << 2,
    _Reserved1 = 3 << 2,
    _Reserved2 = 4 << 2,
    _Reserved3 = 5 << 2,
    _Reserved4 = 6 << 2,
    _Reserved5 = 7 << 2,
}

#[repr(u8)] // 2 bits
#[derive(Debug, PartialEq)]
pub enum Mutability {
    Mutable = 0,
    Immutable = 1,
    _Versioned = 2,
    _Reserved = 3,
}

// packs the aspects from above into a byte
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct IdentifierKind(pub u8);

impl IdentifierKind {
    pub fn create(
        object_type: ObjectType,
        sharing_policy: SharingPolicy,
        mutability: Mutability,
    ) -> IdentifierKind {
        IdentifierKind(object_type as u8 | sharing_policy as u8 | mutability as u8)
    }

    pub fn object_type(&self) -> ObjectType {
        unsafe { std::mem::transmute::<_, _>(self.0 & 0b11100000) }
    }

    pub fn sharing_policy(&self) -> SharingPolicy {
        unsafe { std::mem::transmute::<_, _>(self.0 & 0b00011100) }
    }

    pub fn mutability(&self) -> Mutability {
        unsafe { std::mem::transmute::<_, _>(self.0 & 0b00000011) }
    }

    pub fn components(&self) -> (ObjectType, SharingPolicy, Mutability) {
        (self.object_type(), self.sharing_policy(), self.mutability())
    }
}
