use std::mem;
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};

macro_rules! const_assert_eq {
    ($x:expr, $y:expr) => {
        const _: [(); $x] = [(); $y];
    };
}

pub const PAGE_SIZE_U16: u16 = 4096;
pub const PAGE_SIZE: usize = PAGE_SIZE_U16 as usize;

#[derive(Clone, Immutable, IntoBytes, KnownLayout, TryFromBytes)]
#[repr(u8)]
pub enum PageKind {
    Uber = 0,
    BTreeBranch = 10,
    BTreeLeaf = 20,
}

#[derive(Clone, Immutable, IntoBytes, KnownLayout, TryFromBytes)]
#[repr(transparent)]
pub struct PagePointer(pub u32);

const_assert_eq!(mem::size_of::<PagePointer>(), 4);

impl PagePointer {
    pub const NULL: Self = Self(0);
}

#[derive(Clone, Immutable, IntoBytes, KnownLayout, TryFromBytes)]
#[repr(transparent)]
pub struct PageChecksum(pub [u8; 32]);

impl PageChecksum {
    pub const NULL: Self = Self([0; _]);
}

#[derive(Clone, Immutable, IntoBytes, KnownLayout, TryFromBytes)]
#[repr(C)]
pub struct UberPage {
    pub magic: [u8; 16],
    pub root: PagePointer,
    pub page_size: u16,
    pub _padding: [u8; 4073],
    pub kind: PageKind,
}

const _: () = {
    const MAGIC_OFFSET: usize = 0;
    const_assert_eq!(mem::offset_of!(UberPage, magic), MAGIC_OFFSET);

    const ROOT_OFFSET: usize = MAGIC_OFFSET + mem::size_of::<[u8; 16]>();
    const_assert_eq!(mem::offset_of!(UberPage, root), ROOT_OFFSET);

    const PAGE_SIZE_OFFSET: usize = ROOT_OFFSET + mem::size_of::<PagePointer>();
    const_assert_eq!(mem::offset_of!(UberPage, page_size), PAGE_SIZE_OFFSET);

    const PADDING_OFFSET: usize = PAGE_SIZE_OFFSET + mem::size_of::<u16>();
    const_assert_eq!(mem::offset_of!(UberPage, _padding), PADDING_OFFSET);

    const KIND_OFFSET: usize = PAGE_SIZE - mem::size_of::<PageKind>();
    const_assert_eq!(mem::offset_of!(UberPage, kind), KIND_OFFSET);

    const_assert_eq!(mem::size_of::<UberPage>(), PAGE_SIZE);
};

impl UberPage {
    pub const MAGIC: [u8; 16] = *b"Rook format 0\0  ";
    pub const PAGE_SIZE: u16 = PAGE_SIZE_U16;
    pub const KIND: PageKind = PageKind::Uber;
}

impl Default for UberPage {
    fn default() -> Self {
        Self {
            magic: Self::MAGIC,
            page_size: Self::PAGE_SIZE,
            root: PagePointer::NULL,
            _padding: [0; _],
            kind: Self::KIND,
        }
    }
}

const BRANCH_KEYS_CAPACITY: usize = 92;

#[derive(Clone, Immutable, IntoBytes, KnownLayout, TryFromBytes)]
#[repr(C)]
pub struct BTreeBranchPage {
    pub keys: [[u8; 8]; BRANCH_KEYS_CAPACITY],
    pub children: [PagePointer; BRANCH_KEYS_CAPACITY + 1],
    pub checksums: [PageChecksum; BRANCH_KEYS_CAPACITY + 1],
    pub keys_len: u16,
    pub _padding: [u8; 9],
    pub kind: PageKind,
}

const _: () = {
    const KEYS_OFFSET: usize = 0;
    const_assert_eq!(mem::offset_of!(BTreeBranchPage, keys), KEYS_OFFSET);

    const CHILDREN_OFFSET: usize = KEYS_OFFSET + mem::size_of::<[[u8; 8]; BRANCH_KEYS_CAPACITY]>();
    const_assert_eq!(mem::offset_of!(BTreeBranchPage, children), CHILDREN_OFFSET);

    const CHECKSUMS_OFFSET: usize =
        CHILDREN_OFFSET + mem::size_of::<[PagePointer; BRANCH_KEYS_CAPACITY + 1]>();
    const_assert_eq!(
        mem::offset_of!(BTreeBranchPage, checksums),
        CHECKSUMS_OFFSET
    );

    const KEYS_LEN_OFFSET: usize =
        CHECKSUMS_OFFSET + mem::size_of::<[PageChecksum; BRANCH_KEYS_CAPACITY + 1]>();
    const_assert_eq!(mem::offset_of!(BTreeBranchPage, keys_len), KEYS_LEN_OFFSET);

    const PADDING_OFFSET: usize = KEYS_LEN_OFFSET + mem::size_of::<u16>();
    const_assert_eq!(mem::offset_of!(BTreeBranchPage, _padding), PADDING_OFFSET);

    const KIND_OFFSET: usize = PAGE_SIZE - mem::size_of::<PageKind>();
    const_assert_eq!(mem::offset_of!(BTreeBranchPage, kind), KIND_OFFSET);

    const_assert_eq!(mem::size_of::<BTreeBranchPage>(), PAGE_SIZE);
};

impl BTreeBranchPage {
    pub const KIND: PageKind = PageKind::BTreeBranch;
}

impl Default for BTreeBranchPage {
    fn default() -> Self {
        Self {
            keys: [[0; _]; _],
            children: [PagePointer::NULL; _],
            checksums: [PageChecksum::NULL; _],
            keys_len: 0,
            _padding: [0; _],
            kind: Self::KIND,
        }
    }
}

#[derive(Clone, Immutable, IntoBytes, KnownLayout, TryFromBytes)]
#[repr(C)]
pub struct BTreeLeafPage {
    pub keys_len: u16,
    pub _padding: [u8; 4093],
    pub kind: PageKind,
}

const _: () = {
    const KEYS_LEN_OFFSET: usize = 0;
    const_assert_eq!(mem::offset_of!(BTreeLeafPage, keys_len), KEYS_LEN_OFFSET);

    const PADDING_OFFSET: usize = KEYS_LEN_OFFSET + mem::size_of::<u16>();
    const_assert_eq!(mem::offset_of!(BTreeLeafPage, _padding), PADDING_OFFSET);

    const KIND_OFFSET: usize = PAGE_SIZE - mem::size_of::<PageKind>();
    const_assert_eq!(mem::offset_of!(BTreeLeafPage, kind), KIND_OFFSET);

    const_assert_eq!(mem::size_of::<BTreeLeafPage>(), PAGE_SIZE);
};

impl BTreeLeafPage {
    pub const KIND: PageKind = PageKind::BTreeLeaf;
}

impl Default for BTreeLeafPage {
    fn default() -> Self {
        Self {
            keys_len: 0,
            _padding: [0; _],
            kind: Self::KIND,
        }
    }
}
