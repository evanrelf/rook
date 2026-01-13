use std::{
    io::Write as _,
    mem::{offset_of, size_of},
};
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

const_assert_eq!(size_of::<PagePointer>(), 4);

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
    const_assert_eq!(offset_of!(UberPage, magic), MAGIC_OFFSET);

    const ROOT_OFFSET: usize = MAGIC_OFFSET + size_of::<[u8; 16]>();
    const_assert_eq!(offset_of!(UberPage, root), ROOT_OFFSET);

    const PAGE_SIZE_OFFSET: usize = ROOT_OFFSET + size_of::<PagePointer>();
    const_assert_eq!(offset_of!(UberPage, page_size), PAGE_SIZE_OFFSET);

    const PADDING_OFFSET: usize = PAGE_SIZE_OFFSET + size_of::<u16>();
    const_assert_eq!(offset_of!(UberPage, _padding), PADDING_OFFSET);

    const KIND_OFFSET: usize = PAGE_SIZE - size_of::<PageKind>();
    const_assert_eq!(offset_of!(UberPage, kind), KIND_OFFSET);

    const_assert_eq!(size_of::<UberPage>(), PAGE_SIZE);
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

const BRANCH_CAPACITY: usize = 40;

#[derive(Clone, Immutable, IntoBytes, KnownLayout, TryFromBytes)]
#[repr(C)]
pub struct BTreeBranchPage {
    pub keys: [[u8; 64]; BRANCH_CAPACITY],
    pub children: [PagePointer; BRANCH_CAPACITY + 1],
    pub checksums: [PageChecksum; BRANCH_CAPACITY + 1],
    pub keys_len: u16,
    pub _padding: [u8; 57],
    pub kind: PageKind,
}

const _: () = {
    const KEYS_OFFSET: usize = 0;
    const_assert_eq!(offset_of!(BTreeBranchPage, keys), KEYS_OFFSET);

    const CHILDREN_OFFSET: usize = KEYS_OFFSET + size_of::<[[u8; 64]; BRANCH_CAPACITY]>();
    const_assert_eq!(offset_of!(BTreeBranchPage, children), CHILDREN_OFFSET);

    const CHECKSUMS_OFFSET: usize =
        CHILDREN_OFFSET + size_of::<[PagePointer; BRANCH_CAPACITY + 1]>();
    const_assert_eq!(offset_of!(BTreeBranchPage, checksums), CHECKSUMS_OFFSET);

    const KEYS_LEN_OFFSET: usize =
        CHECKSUMS_OFFSET + size_of::<[PageChecksum; BRANCH_CAPACITY + 1]>();
    const_assert_eq!(offset_of!(BTreeBranchPage, keys_len), KEYS_LEN_OFFSET);

    const PADDING_OFFSET: usize = KEYS_LEN_OFFSET + size_of::<u16>();
    const_assert_eq!(offset_of!(BTreeBranchPage, _padding), PADDING_OFFSET);

    const KIND_OFFSET: usize = PAGE_SIZE - size_of::<PageKind>();
    const_assert_eq!(offset_of!(BTreeBranchPage, kind), KIND_OFFSET);

    const_assert_eq!(size_of::<BTreeBranchPage>(), PAGE_SIZE);
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

const LEAF_CAPACITY: usize = 31;

#[derive(Clone, Immutable, IntoBytes, KnownLayout, TryFromBytes)]
#[repr(C)]
pub struct BTreeLeafPage {
    pub keys: [[u8; 64]; LEAF_CAPACITY],
    pub values: [[u8; 64]; LEAF_CAPACITY],
    pub length: u16,
    pub _padding: [u8; 125],
    pub kind: PageKind,
}

const _: () = {
    const KEYS_OFFSET: usize = 0;
    const_assert_eq!(offset_of!(BTreeLeafPage, keys), KEYS_OFFSET);

    const VALUES_OFFSET: usize = KEYS_OFFSET + size_of::<[[u8; 64]; LEAF_CAPACITY]>();
    const_assert_eq!(offset_of!(BTreeLeafPage, values), VALUES_OFFSET);

    const LENGTH_OFFSET: usize = VALUES_OFFSET + size_of::<[[u8; 64]; LEAF_CAPACITY]>();
    const_assert_eq!(offset_of!(BTreeLeafPage, length), LENGTH_OFFSET);

    const PADDING_OFFSET: usize = LENGTH_OFFSET + size_of::<u16>();
    const_assert_eq!(offset_of!(BTreeLeafPage, _padding), PADDING_OFFSET);

    const KIND_OFFSET: usize = PAGE_SIZE - size_of::<PageKind>();
    const_assert_eq!(offset_of!(BTreeLeafPage, kind), KIND_OFFSET);

    const_assert_eq!(size_of::<BTreeLeafPage>(), PAGE_SIZE);
};

impl BTreeLeafPage {
    pub const KIND: PageKind = PageKind::BTreeLeaf;
}

impl Default for BTreeLeafPage {
    fn default() -> Self {
        Self {
            keys: [[0; _]; _],
            values: [[0; _]; _],
            length: 0,
            _padding: [0; _],
            kind: Self::KIND,
        }
    }
}

pub enum PageRef<'a> {
    Uber(&'a UberPage),
    BTreeBranch(&'a BTreeBranchPage),
    BTreeLeaf(&'a BTreeLeafPage),
}

pub fn push_page<'a>(db: &mut Vec<u8>, page: PageRef<'a>) -> PagePointer {
    let page_index = db.len() / PAGE_SIZE;
    let pointer = PagePointer(u32::try_from(page_index).unwrap());
    db.resize(db.len() + PAGE_SIZE, 0);
    let byte_offset = page_index * PAGE_SIZE;
    let page_bytes = match page {
        PageRef::Uber(page) => page.as_bytes(),
        PageRef::BTreeBranch(page) => page.as_bytes(),
        PageRef::BTreeLeaf(page) => page.as_bytes(),
    };
    (&mut db[byte_offset..byte_offset + PAGE_SIZE])
        .write_all(page_bytes)
        .unwrap();
    pointer
}

pub fn get_page<'a>(db: &'a [u8], pointer: &PagePointer) -> PageRef<'a> {
    let offset = usize::try_from(pointer.0).expect("usize >= 32 bits") * PAGE_SIZE;
    let page_bytes = &db[offset..offset + PAGE_SIZE];
    let page_kind = PageKind::try_ref_from_bytes(&page_bytes[PAGE_SIZE - 1..PAGE_SIZE]).unwrap();
    match page_kind {
        PageKind::Uber => PageRef::Uber(UberPage::try_ref_from_bytes(page_bytes).unwrap()),
        PageKind::BTreeBranch => {
            PageRef::BTreeBranch(BTreeBranchPage::try_ref_from_bytes(page_bytes).unwrap())
        }
        PageKind::BTreeLeaf => {
            PageRef::BTreeLeaf(BTreeLeafPage::try_ref_from_bytes(page_bytes).unwrap())
        }
    }
}

pub enum PageMut<'a> {
    Uber(&'a mut UberPage),
    BTreeBranch(&'a mut BTreeBranchPage),
    BTreeLeaf(&'a mut BTreeLeafPage),
}

pub fn get_page_mut<'a>(db: &'a mut [u8], pointer: &PagePointer) -> PageMut<'a> {
    let offset = usize::try_from(pointer.0).expect("usize >= 32 bits") * PAGE_SIZE;
    let page_bytes = &mut db[offset..offset + PAGE_SIZE];
    let page_kind = PageKind::try_ref_from_bytes(&page_bytes[PAGE_SIZE - 1..PAGE_SIZE]).unwrap();
    match page_kind {
        PageKind::Uber => PageMut::Uber(UberPage::try_mut_from_bytes(page_bytes).unwrap()),
        PageKind::BTreeBranch => {
            PageMut::BTreeBranch(BTreeBranchPage::try_mut_from_bytes(page_bytes).unwrap())
        }
        PageKind::BTreeLeaf => {
            PageMut::BTreeLeaf(BTreeLeafPage::try_mut_from_bytes(page_bytes).unwrap())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut db = Vec::new();
        let uber_page_pointer = push_page(&mut db, PageRef::Uber(&UberPage::default()));
        let leaf_page_pointer = push_page(&mut db, PageRef::BTreeLeaf(&BTreeLeafPage::default()));
        if let PageMut::Uber(uber_page) = get_page_mut(&mut db, &uber_page_pointer) {
            uber_page.root = leaf_page_pointer;
        } else {
            unreachable!()
        };
        if let PageRef::Uber(uber_page) = get_page(&db, &uber_page_pointer) {
            if let PageRef::BTreeLeaf(_leaf_page) = get_page(&db, &uber_page.root) {
                // nice
            } else {
                unreachable!()
            }
        } else {
            unreachable!()
        }
    }
}
