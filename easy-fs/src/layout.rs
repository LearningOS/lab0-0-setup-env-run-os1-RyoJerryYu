use alloc::{sync::Arc, vec::Vec};

use crate::{block_cache::get_block_cache, block_dev::BlockDevice, BLOCK_SZ};

/// Magic number for sanity check
const EFS_MAGIC: u32 = 0x3b800001;
/// The max number of direct inodes
const INODE_DIRECT_COUNT: usize = 28;
/// The max length of inode name
const NAME_LENGTH_LIMIT: usize = 27;
/// The max number of indirect1 inodes
const INODE_INDIRECT1_COUNT: usize = BLOCK_SZ / 4;
/// The max number of indirect2 inodes
const INODE_INDIRECT2_COUNT: usize = INODE_INDIRECT1_COUNT * INODE_INDIRECT1_COUNT;
/// The upper bound of direct inode index
const DIRECT_BOUND: usize = INODE_DIRECT_COUNT;
/// The upper bound of indirect1 inode index
const INDIRECT1_BOUND: usize = DIRECT_BOUND + INODE_INDIRECT1_COUNT;
/// The upper bound of indirect2 inode indexs
#[allow(unused)]
const INDIRECT2_BOUND: usize = INDIRECT1_BOUND + INODE_INDIRECT2_COUNT;

#[repr(C)]
pub struct SuperBlock {
    magic: u32,
    pub total_blocks: u32,
    pub inode_bitmap_blocks: u32,
    pub inode_area_blocks: u32,
    pub data_bitmap_blocks: u32,
    pub data_area_blocks: u32,
}

impl SuperBlock {
    pub fn initialize(
        &mut self,
        total_blocks: u32,
        inode_bitmap_blocks: u32,
        inode_area_blocks: u32,
        data_bitmap_blocks: u32,
        data_area_blocks: u32,
    ) {
        self.magic = EFS_MAGIC;
        self.total_blocks = total_blocks;
        self.inode_bitmap_blocks = inode_bitmap_blocks;
        self.inode_area_blocks = inode_area_blocks;
        self.data_bitmap_blocks = data_bitmap_blocks;
        self.data_area_blocks = data_area_blocks;
    }

    pub fn is_valid(&self) -> bool {
        self.magic == EFS_MAGIC
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskInodeType {
    File,
    Directory,
}

/// A indirect block
type IndirectBlock = [u32; BLOCK_SZ / 4];
/// A data block
type DataBlock = [u8; BLOCK_SZ];

/// DiskInode is the index node stored in disk
/// 索引节点
#[repr(C)]
pub struct DiskInode {
    pub size: u32,
    /// every element in direct point to a data block
    /// so it can point to 28 * 512 = 14KB data
    pub direct: [u32; INODE_DIRECT_COUNT],
    /// indirect1 is a first level index,
    /// every u32 points to a data block,
    /// so it can point to 128 * 512 = 64KB data
    pub indirect1: u32,
    /// indirect2 is a second level index,
    /// every u32 points to a first level index,
    /// so it can point to 128 * 128 * 512 = 8MB data
    pub indirect2: u32,
    type_: DiskInodeType,
}

impl DiskInode {
    pub fn initialize(&mut self, type_: DiskInodeType) {
        self.size = 0;
        self.type_ = type_;
        self.direct.iter_mut().for_each(|x| *x = 0);
        self.indirect1 = 0;
        self.indirect2 = 0;
        self.type_ = type_;
    }
    pub fn is_dir(&self) -> bool {
        self.type_ == DiskInodeType::Directory
    }
    pub fn is_file(&self) -> bool {
        self.type_ == DiskInodeType::File
    }

    /// Get the number of data blocks that the inode occupies
    pub fn data_blocks(&self) -> u32 {
        DiskInode::_data_blocks(self.size)
    }
    // ceil(size / BLOCK_SZ)
    pub fn _data_blocks(size: u32) -> u32 {
        (size + BLOCK_SZ as u32 - 1) / BLOCK_SZ as u32
    }

    // how many blocks the inode occupies, including data blocks and indirect blocks
    pub fn total_blocks(size: u32) -> u32 {
        let data_blocks = Self::_data_blocks(size) as usize;
        let mut total = data_blocks as usize;
        // indirect1
        if data_blocks > INODE_DIRECT_COUNT {
            total += 1; // include indirect1
        }
        // indirect2
        if data_blocks > INDIRECT1_BOUND {
            total += 1; // include indirect2

            // include indirect1s in indirect2
            // ceil((data_blocks - INDIRECT1_BOUND) / INODE_INDIRECT1_COUNT)
            total +=
                (data_blocks - INDIRECT1_BOUND + INODE_INDIRECT1_COUNT - 1) / INODE_INDIRECT1_COUNT;
        }
        total as u32
    }

    /// Get the number of data blocks that have to be allocated given the new size of data
    pub fn blocks_num_needed(&self, new_size: u32) -> u32 {
        assert!(new_size >= self.size);
        Self::total_blocks(new_size) - Self::total_blocks(self.size)
    }

    /// Get the block_id_th block of the file
    pub fn get_block_id(&self, inner_id: u32, block_device: &Arc<dyn BlockDevice>) -> u32 {
        let inner_id = inner_id as usize;
        if inner_id < INODE_DIRECT_COUNT {
            // direct
            self.direct[inner_id]
        } else if inner_id < INODE_DIRECT_COUNT + INODE_INDIRECT1_COUNT {
            // in indirect1
            get_block_cache(self.indirect1 as usize, block_device.clone())
                .lock()
                .read(0, |indirect_block: &IndirectBlock| {
                    // count start from INODE_DIRECT_COUNT in indirect1
                    indirect_block[inner_id - INODE_DIRECT_COUNT]
                })
        } else {
            let last = inner_id - INDIRECT1_BOUND; // count start from INDIRECT1_BOUND in indirect2
            let indirect1 = get_block_cache(self.indirect2 as usize, block_device.clone())
                .lock() // the block that indirect2 point to
                .read(0, |indirect2: &IndirectBlock| {
                    indirect2[last / INODE_INDIRECT1_COUNT] // the pointer to indirect1
                });
            get_block_cache(indirect1 as usize, block_device.clone())
                .lock()
                .read(0, |indirect1: &IndirectBlock| {
                    indirect1[last % INODE_INDIRECT1_COUNT] // the pointer to data block
                })
        }
    }

    /// Increate the size of the inode
    pub fn increase_size(
        &mut self,
        new_size: u32,        // the new size of the file
        new_blocks: Vec<u32>, // how many blocks to allocate
        block_device: &Arc<dyn BlockDevice>,
    ) {
        todo!()
    }
}
