use std::{
    io::Write as _,
    mem::{offset_of, size_of},
};
use twox_hash::XxHash3_64;
use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes};

macro_rules! const_assert_eq {
    ($x:expr, $y:expr) => {
        const _: [(); $y] = [(); $x];
    };
}

pub const MAGIC: [u8; 16] = *b"Rook format 0\0  ";

pub const PAGE_SIZE_U16: u16 = 4096;
pub const PAGE_SIZE: usize = PAGE_SIZE_U16 as usize;

#[derive(Clone, Immutable, IntoBytes, KnownLayout, TryFromBytes)]
#[repr(u8)]
pub enum PageKind {
    Super = 0,
    BTreeBranch = 1,
    BTreeLeaf = 2,
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
pub struct PageChecksum(pub u64);

impl PageChecksum {
    pub const NULL: Self = Self(0);
}

#[derive(Clone, Default, Immutable, IntoBytes, KnownLayout, TryFromBytes)]
#[repr(transparent)]
pub struct SuperPageGeneration(pub u64);

pub const SUPER_PAGE_COUNT: usize = 2;

const _: () = assert!(
    SUPER_PAGE_COUNT >= 2,
    "Must have at least 2 super pages for atomic writes"
);

pub const SUPER_PAGE_STRIDE: usize = PAGE_SIZE * 2;

const _: () =
    assert!(SUPER_PAGE_STRIDE >= PAGE_SIZE && SUPER_PAGE_STRIDE.is_multiple_of(PAGE_SIZE));

pub const SUPER_PAGE_GAP: usize = SUPER_PAGE_STRIDE - PAGE_SIZE;

#[derive(Clone, Immutable, IntoBytes, KnownLayout, TryFromBytes)]
#[repr(C)]
pub struct SuperPage {
    pub magic: [u8; 16],
    pub generation: SuperPageGeneration,
    pub root_checksum: PageChecksum,
    pub root_pointer: PagePointer,
    pub page_size: u16,
    pub _padding: [u8; 4057],
    pub kind: PageKind,
}

const _: () = {
    const MAGIC_OFFSET: usize = 0;
    const_assert_eq!(offset_of!(SuperPage, magic), MAGIC_OFFSET);

    const GENERATION_OFFSET: usize = MAGIC_OFFSET + size_of::<[u8; 16]>();
    const_assert_eq!(offset_of!(SuperPage, generation), GENERATION_OFFSET);

    const ROOT_CHECKSUM_OFFSET: usize = GENERATION_OFFSET + size_of::<SuperPageGeneration>();
    const_assert_eq!(offset_of!(SuperPage, root_checksum), ROOT_CHECKSUM_OFFSET);

    const ROOT_POINTER_OFFSET: usize = ROOT_CHECKSUM_OFFSET + size_of::<PageChecksum>();
    const_assert_eq!(offset_of!(SuperPage, root_pointer), ROOT_POINTER_OFFSET);

    const PAGE_SIZE_OFFSET: usize = ROOT_POINTER_OFFSET + size_of::<PagePointer>();
    const_assert_eq!(offset_of!(SuperPage, page_size), PAGE_SIZE_OFFSET);

    const PADDING_OFFSET: usize = PAGE_SIZE_OFFSET + size_of::<u16>();
    const_assert_eq!(offset_of!(SuperPage, _padding), PADDING_OFFSET);

    const KIND_OFFSET: usize = PAGE_SIZE - size_of::<PageKind>();
    const_assert_eq!(offset_of!(SuperPage, kind), KIND_OFFSET);

    const_assert_eq!(size_of::<SuperPage>(), PAGE_SIZE);
};

impl Default for SuperPage {
    fn default() -> Self {
        Self {
            magic: MAGIC,
            generation: SuperPageGeneration::default(),
            page_size: PAGE_SIZE_U16,
            root_pointer: PagePointer::NULL,
            root_checksum: PageChecksum::NULL,
            _padding: [0; _],
            kind: PageKind::Super,
        }
    }
}

const BRANCH_CAPACITY: usize = 53;

#[derive(Clone, Immutable, IntoBytes, KnownLayout, TryFromBytes)]
#[repr(C)]
pub struct BTreeBranchPage {
    pub keys: [[u8; 64]; BRANCH_CAPACITY],
    pub child_checksums: [PageChecksum; BRANCH_CAPACITY + 1],
    pub child_pointers: [PagePointer; BRANCH_CAPACITY + 1],
    pub keys_len: u16,
    pub _padding: [u8; 53],
    pub kind: PageKind,
}

const _: () = {
    const KEYS_OFFSET: usize = 0;
    const_assert_eq!(offset_of!(BTreeBranchPage, keys), KEYS_OFFSET);

    const CHILD_CHECKSUMS_OFFSET: usize = KEYS_OFFSET + size_of::<[[u8; 64]; BRANCH_CAPACITY]>();
    const_assert_eq!(
        offset_of!(BTreeBranchPage, child_checksums),
        CHILD_CHECKSUMS_OFFSET
    );

    const CHILD_POINTERS_OFFSET: usize =
        CHILD_CHECKSUMS_OFFSET + size_of::<[PageChecksum; BRANCH_CAPACITY + 1]>();
    const_assert_eq!(
        offset_of!(BTreeBranchPage, child_pointers),
        CHILD_POINTERS_OFFSET
    );

    const KEYS_LEN_OFFSET: usize =
        CHILD_POINTERS_OFFSET + size_of::<[PagePointer; BRANCH_CAPACITY + 1]>();
    const_assert_eq!(offset_of!(BTreeBranchPage, keys_len), KEYS_LEN_OFFSET);

    const PADDING_OFFSET: usize = KEYS_LEN_OFFSET + size_of::<u16>();
    const_assert_eq!(offset_of!(BTreeBranchPage, _padding), PADDING_OFFSET);

    const KIND_OFFSET: usize = PAGE_SIZE - size_of::<PageKind>();
    const_assert_eq!(offset_of!(BTreeBranchPage, kind), KIND_OFFSET);

    const_assert_eq!(size_of::<BTreeBranchPage>(), PAGE_SIZE);
};

impl Default for BTreeBranchPage {
    fn default() -> Self {
        Self {
            keys: [[0; _]; _],
            child_pointers: [PagePointer::NULL; _],
            child_checksums: [PageChecksum::NULL; _],
            keys_len: 0,
            _padding: [0; _],
            kind: PageKind::BTreeBranch,
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

impl Default for BTreeLeafPage {
    fn default() -> Self {
        Self {
            keys: [[0; _]; _],
            values: [[0; _]; _],
            length: 0,
            _padding: [0; _],
            kind: PageKind::BTreeLeaf,
        }
    }
}

/**************************************************************************************************/

pub struct Database {
    bytes: Vec<u8>,
}

impl Database {
    pub fn new() -> Self {
        let mut db = Self { bytes: Vec::new() };

        // Create super pages
        let super_page = PageRef::Super(&SuperPage::default());
        let mut super_page_pointers = Vec::with_capacity(SUPER_PAGE_COUNT);
        super_page_pointers.push(db.push_page(&super_page));
        for _ in 1..SUPER_PAGE_COUNT {
            db.bytes.resize(db.bytes.len() + SUPER_PAGE_GAP, 0);
            super_page_pointers.push(db.push_page(&super_page));
        }

        // Create root leaf page
        let leaf_page = BTreeLeafPage::default();
        let leaf_page_ref = PageRef::BTreeLeaf(&leaf_page);
        let leaf_page_pointer = db.push_page(&leaf_page_ref);
        let leaf_page_checksum = PageChecksum(XxHash3_64::oneshot(leaf_page.as_bytes()));

        // Update all(?) super pages to point to leaf page
        for super_page_pointer in &super_page_pointers {
            let PageMut::Super(super_page) = db.get_page_mut(super_page_pointer) else {
                unreachable!()
            };
            super_page.root_pointer = leaf_page_pointer.clone();
            super_page.root_checksum = leaf_page_checksum.clone();
            super_page.generation.0 += 1;
        }

        db
    }

    pub fn push_page<'a>(&mut self, page: &PageRef<'a>) -> PagePointer {
        let page_index = self.bytes.len() / PAGE_SIZE;
        let pointer = PagePointer(u32::try_from(page_index).unwrap());
        self.bytes.resize(self.bytes.len() + PAGE_SIZE, 0);
        let byte_offset = page_index * PAGE_SIZE;
        let page_bytes = match page {
            PageRef::Super(page) => page.as_bytes(),
            PageRef::BTreeBranch(page) => page.as_bytes(),
            PageRef::BTreeLeaf(page) => page.as_bytes(),
        };
        (&mut self.bytes[byte_offset..byte_offset + PAGE_SIZE])
            .write_all(page_bytes)
            .unwrap();
        pointer
    }

    pub fn get_page<'a>(&'a self, pointer: &PagePointer) -> PageRef<'a> {
        let offset = usize::try_from(pointer.0).expect("usize >= 32 bits") * PAGE_SIZE;
        let page_bytes = &self.bytes[offset..offset + PAGE_SIZE];
        let page_kind =
            PageKind::try_ref_from_bytes(&page_bytes[PAGE_SIZE - 1..PAGE_SIZE]).unwrap();
        match page_kind {
            PageKind::Super => PageRef::Super(SuperPage::try_ref_from_bytes(page_bytes).unwrap()),
            PageKind::BTreeBranch => {
                PageRef::BTreeBranch(BTreeBranchPage::try_ref_from_bytes(page_bytes).unwrap())
            }
            PageKind::BTreeLeaf => {
                PageRef::BTreeLeaf(BTreeLeafPage::try_ref_from_bytes(page_bytes).unwrap())
            }
        }
    }

    pub fn get_page_mut<'a>(&'a mut self, pointer: &PagePointer) -> PageMut<'a> {
        let offset = usize::try_from(pointer.0).expect("usize >= 32 bits") * PAGE_SIZE;
        let page_bytes = &mut self.bytes[offset..offset + PAGE_SIZE];
        let page_kind =
            PageKind::try_ref_from_bytes(&page_bytes[PAGE_SIZE - 1..PAGE_SIZE]).unwrap();
        match page_kind {
            PageKind::Super => PageMut::Super(SuperPage::try_mut_from_bytes(page_bytes).unwrap()),
            PageKind::BTreeBranch => {
                PageMut::BTreeBranch(BTreeBranchPage::try_mut_from_bytes(page_bytes).unwrap())
            }
            PageKind::BTreeLeaf => {
                PageMut::BTreeLeaf(BTreeLeafPage::try_mut_from_bytes(page_bytes).unwrap())
            }
        }
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

pub enum PageRef<'a> {
    Super(&'a SuperPage),
    BTreeBranch(&'a BTreeBranchPage),
    BTreeLeaf(&'a BTreeLeafPage),
}

pub enum PageMut<'a> {
    Super(&'a mut SuperPage),
    BTreeBranch(&'a mut BTreeBranchPage),
    BTreeLeaf(&'a mut BTreeLeafPage),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let db = Database::new();
        let super_page_pointer = PagePointer(0);
        if let PageRef::Super(super_page) = db.get_page(&super_page_pointer) {
            if let PageRef::BTreeLeaf(_leaf_page) = db.get_page(&super_page.root_pointer) {
                // nice
            } else {
                unreachable!()
            }
        } else {
            unreachable!()
        }
    }
}
