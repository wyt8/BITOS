use alloc::{sync::Arc, vec, vec::Vec};
use bitvec::access;
use core::time::Duration;

use align_ext::AlignExt;
use aster_block::{id::Bid, BlockDevice};
use aster_rights::Full;
use ostd::mm::VmIo;

use crate::{
    fs::utils::{
        CStr256, DirentVisitor, Extension, FallocMode, FileSystem, FsFlags, Inode, InodeMode,
        InodeType, IoctlCmd, Metadata, MknodType, PageCache, SuperBlock, XattrName, XattrNamespace,
        XattrSetFlags,
    },
    prelude::*,
    process::{Gid, Uid},
    util::MultiWrite,
    vm::vmo::Vmo,
};

const BLOCK_SIZE: usize = another_ext4::BLOCK_SIZE;
const ROOT_INO: usize = 2;
const NAME_MAX: usize = 255;

#[derive(Debug)]
pub struct BlockDeviceWrapper {
    inner: Arc<dyn BlockDevice>,
}

impl another_ext4::BlockDevice for BlockDeviceWrapper {
    /// Read a block from disk.
    fn read_block(&self, block_id: another_ext4::PBlockId) -> another_ext4::Block {
        let offset = (block_id * BLOCK_SIZE as u64) as usize;
        let mut buf = vec![0u8; BLOCK_SIZE];
        self.inner.read_bytes(offset, buf.as_mut());
        another_ext4::Block {
            id: block_id,
            data: buf.try_into().unwrap(),
        }
    }

    /// Write a block to disk.
    fn write_block(&self, block: &another_ext4::Block) {
        let offset = (block.id * BLOCK_SIZE as u64) as usize;
        self.inner.write_bytes(offset, &block.data);
    }
}

pub struct Ext4Inode {
    fs: Weak<Ext4>,
    inner: RwMutex<Ext4InodeInner>,
}

pub struct Ext4InodeInner {
    inode: another_ext4::InodeRef,
}

impl From<another_ext4::FileType> for InodeType {
    fn from(file_type: another_ext4::FileType) -> Self {
        match file_type {
            another_ext4::FileType::Unknown => unimplemented!(),
            another_ext4::FileType::RegularFile => InodeType::File,
            another_ext4::FileType::Directory => InodeType::Dir,
            another_ext4::FileType::CharacterDev => InodeType::CharDevice,
            another_ext4::FileType::BlockDev => InodeType::BlockDevice,
            another_ext4::FileType::Fifo => InodeType::NamedPipe,
            another_ext4::FileType::Socket => InodeType::Socket,
            another_ext4::FileType::SymLink => InodeType::SymLink,
        }
    }
}

impl From<InodeType> for another_ext4::FileType {
    fn from(value: InodeType) -> Self {
        match value {
            InodeType::File => Self::RegularFile,
            InodeType::Dir => Self::Directory,
            InodeType::CharDevice => Self::CharacterDev,
            InodeType::BlockDevice => Self::BlockDev,
            InodeType::NamedPipe => Self::Fifo,
            InodeType::Socket => Self::Socket,
            InodeType::SymLink => Self::SymLink,
        }
    }
}

impl From<another_ext4::InodeMode> for InodeMode {
    fn from(value: another_ext4::InodeMode) -> Self {
        Self::from_bits_truncate(value.bits() as _)
    }
}

impl From<InodeMode> for another_ext4::InodeMode {
    fn from(mode: InodeMode) -> Self {
        Self::from_bits_truncate(mode.bits() as _)
    }
}

impl Ext4Inode {
    pub fn ino(&self) -> u32 {
        self.inner.read().inode.id
    }

    pub fn inode_type(&self) -> InodeType {
        self.inner.read().inode.inode.file_type().into()
    }

    pub fn device_id(&self) -> u64 {
        if self.inode_type() != InodeType::BlockDevice && self.inode_type() != InodeType::CharDevice
        {
            return 0;
        }

        // let mut device_id: u64 = 0;
        // device_id.as_bytes_mut().copy_from_slice(
        //     &self.inner.read().inode.inode.block.as_bytes()[..core::mem::size_of::<u64>()],
        // );
        // device_id
        0
    }

    pub fn fs(&self) -> Arc<Ext4> {
        self.fs.upgrade().unwrap()
    }

    pub fn file_size(&self) -> usize {
        self.inner.read().inode.inode.size() as _
    }
}

impl Ext4InodeInner {
    pub fn resize(&mut self, new_size: usize) -> Result<()> {
        // self.page_cache.resize(new_size)?;
        self.inode.inode.set_size(new_size as _);
        Ok(())
    }
}

impl Inode for Ext4Inode {
    fn size(&self) -> usize {
        self.file_size()
    }

    fn resize(&self, new_size: usize) -> Result<()> {
        let now = now();
        let fs = self.fs();
        fs.inner.setattr(
            self.ino() as _,
            None,
            None,
            None,
            Some(new_size as _),
            Some(now.as_secs() as _),
            Some(now.as_secs() as _),
            Some(now.as_secs() as _),
            None,
        );
        Ok(())
    }

    fn metadata(&self) -> Metadata {
        Metadata {
            dev: 0, // TODO: ID of block device
            ino: self.ino() as _,
            size: self.size() as _,
            blk_size: BLOCK_SIZE,
            blocks: self.inner.read().inode.inode.block_count() as _,
            atime: self.atime(),
            mtime: self.mtime(),
            ctime: self.ctime(),
            type_: self.type_(),
            mode: self.mode().unwrap(),
            nlinks: self.inner.read().inode.inode.link_count() as _,
            uid: self.owner().unwrap(),
            gid: self.group().unwrap(),
            rdev: self.device_id(),
        }
    }

    fn atime(&self) -> Duration {
        Duration::from_secs(self.inner.read().inode.inode.atime() as u64)
    }

    fn set_atime(&self, time: Duration) {
        self.inner
            .write()
            .inode
            .inode
            .set_atime(time.as_secs() as u32);
    }
    /// Returns the last modification time of the file.
    fn mtime(&self) -> Duration {
        Duration::from_secs(self.inner.read().inode.inode.mtime() as u64)
    }

    fn set_mtime(&self, time: Duration) {
        self.inner
            .write()
            .inode
            .inode
            .set_mtime(time.as_secs() as u32);
    }

    fn ctime(&self) -> Duration {
        Duration::from_secs(self.inner.read().inode.inode.ctime() as u64)
    }

    fn set_ctime(&self, time: Duration) {
        self.inner
            .write()
            .inode
            .inode
            .set_ctime(time.as_secs() as u32);
    }

    fn ino(&self) -> u64 {
        self.ino() as _
    }

    fn type_(&self) -> InodeType {
        self.inode_type()
    }

    fn mode(&self) -> Result<InodeMode> {
        Ok(InodeMode::from(self.inner.read().inode.inode.perm()))
    }

    fn set_mode(&self, mode: InodeMode) -> Result<()> {
        self.inner
            .write()
            .inode
            .inode
            .set_mode(another_ext4::InodeMode::from(mode));
        Ok(())
    }

    fn owner(&self) -> Result<Uid> {
        Ok(Uid::new(self.inner.read().inode.inode.uid() as u32))
    }

    fn set_owner(&self, uid: Uid) -> Result<()> {
        self.inner
            .write()
            .inode
            .inode
            .set_uid(Into::<u32>::into(uid) as _);
        Ok(())
    }

    fn group(&self) -> Result<Gid> {
        Ok(Gid::new(self.inner.read().inode.inode.gid() as u32))
    }

    fn set_group(&self, gid: Gid) -> Result<()> {
        self.inner
            .write()
            .inode
            .inode
            .set_gid(Into::<u32>::into(gid) as _);
        Ok(())
    }

    fn page_cache(&self) -> Option<Vmo<Full>> {
        None
    }

    fn read_at(&self, offset: usize, writer: &mut VmWriter) -> Result<usize> {
        self.read_direct_at(offset, writer)
    }

    fn read_direct_at(&self, offset: usize, writer: &mut VmWriter) -> Result<usize> {
        // if self.type_() != InodeType::File {
        //     return_errno!(Errno::EISDIR);
        // }
        // if !is_block_aligned(offset) || !is_block_aligned(writer.avail()) {
        //     return_errno_with_message!(Errno::EINVAL, "not block-aligned");
        // }

        // let (offset, read_len) = {
        //     let file_size = self.size();
        //     let start = file_size.min(offset).align_down(BLOCK_SIZE);
        //     let end = file_size
        //         .min(offset + writer.avail())
        //         .align_down(BLOCK_SIZE);
        //     (start, end - start)
        // };

        // if read_len == 0 {
        //     return Ok(read_len);
        // }

        let file_size = self.size();
        let avail = writer.avail();

        let read_len = (file_size-offset).min(avail);

        let mut read_buf = vec![0u8; read_len];

        self.fs().inner.read(self.ino() as _, offset, &mut read_buf);

        let mut reader = VmReader::from(read_buf.as_slice());

        writer.write(&mut reader);

        
        // for i in 0..read_len {
        //     writer.write_val(&read_buf[i]);
        // }

        self.set_atime(now());

        Ok(read_len)
    }

    fn write_at(&self, offset: usize, reader: &mut VmReader) -> Result<usize> {
        self.write_direct_at(offset, reader)
    }

    fn write_direct_at(&self, offset: usize, reader: &mut VmReader) -> Result<usize> {
        if self.type_() != InodeType::File {
            return_errno!(Errno::EISDIR);
        }

        let write_buf = reader.collect().unwrap();

        self.fs().inner.write(self.ino() as _, offset, &write_buf);
        Ok(write_buf.len())
    }

    fn create(&self, name: &str, type_: InodeType, mode: InodeMode) -> Result<Arc<dyn Inode>> {
        const EXT4_GOOD_OLD_INODE_SIZE: u16 = 128;
        const EXT4_INODE_FLAG_EXTENTS: usize = 0x00080000; /* Inode uses extents */

        let fs = self.fs();

        let mode = another_ext4::InodeMode::from_type_and_perm(
            another_ext4::FileType::from(type_),
            another_ext4::InodeMode::from(mode),
        );

        let child_ino = fs.inner.create(self.ino() as _, name, mode).unwrap();

        let child_inode_ref = fs.inner.read_inode(child_ino);

        Ok(Arc::new(Ext4Inode {
            fs: Arc::downgrade(&fs),
            inner: RwMutex::new(Ext4InodeInner {
                inode: child_inode_ref,
            }),
        }))
    }

    fn mknod(&self, name: &str, mode: InodeMode, type_: MknodType) -> Result<Arc<dyn Inode>> {
        let inode_type = type_.inode_type();
        let inode = match type_ {
            MknodType::CharDeviceNode(dev) | MknodType::BlockDeviceNode(dev) => {
                let inode = self.create(name, inode_type, mode.into())?;
                // inode.set_device_id(dev.id().into()).unwrap();
                inode
            }
            _ => todo!(),
        };

        Ok(inode)
    }

    fn lookup(&self, name: &str) -> Result<Arc<dyn Inode>> {
        let fs = self.fs();

        let ino = fs.inner.lookup(self.ino() as _, name);

        match ino {
            Ok(ino) => {
                let inode_ref = fs.inner.read_inode(ino);
                Ok(Arc::new(Ext4Inode {
                    fs: Arc::downgrade(&fs),
                    inner: RwMutex::new(Ext4InodeInner { inode: inode_ref }),
                }))
            }
            Err(_) => return_errno!(Errno::ENOENT),
        }
    }

    fn readdir_at(&self, offset: usize, visitor: &mut dyn DirentVisitor) -> Result<usize> {
        let offset_read = {
            let try_readdir = |offset_: &mut usize, visitor: &mut dyn DirentVisitor| -> Result<()> {
                let fs = self.fs();
                let dirs = fs.inner.listdir(self.ino() as _).unwrap();
                let mut acc_offset = 0;
                for dir in dirs.iter() {
                    if acc_offset >= offset {
                        let name = dir.name();
                        let ino = dir.inode() as _;
                        let type_ = InodeType::from(dir.file_type());
                        visitor.visit(&name, ino, type_, dir.rec_len as _)?;
                    }
                    acc_offset += dir.rec_len as usize;
                    *offset_ += dir.rec_len as usize;
                }
                Ok(())
            };

            let mut iterate_offset = offset;
            match try_readdir(&mut iterate_offset, visitor) {
                Err(e) if iterate_offset == offset => Err(e),
                _ => Ok(iterate_offset - offset),
            }?
        };
        Ok(offset_read)
    }

    fn link(&self, old: &Arc<dyn Inode>, name: &str) -> Result<()> {
        let old = old
            .downcast_ref::<Ext4Inode>()
            .ok_or_else(|| Error::with_message(Errno::EXDEV, "not same fs"))?;
        let fs = self.fs();
        fs.inner.link(
            old.inner.write().inode.id as _,
            self.inner.write().inode.id as _,
            name,
        );
        Ok(())
    }

    fn unlink(&self, name: &str) -> Result<()> {
        let fs = self.fs();
        fs.inner.unlink(self.ino() as _, name);
        Ok(())
    }

    fn rmdir(&self, name: &str) -> Result<()> {
        let fs = self.fs();
        fs.inner.rmdir(self.ino() as _, name);
        Ok(())
    }

    fn rename(&self, old_name: &str, target: &Arc<dyn Inode>, new_name: &str) -> Result<()> {
        let target = target
            .downcast_ref::<Ext4Inode>()
            .ok_or_else(|| Error::with_message(Errno::EXDEV, "not same fs"))?;
        let fs = self.fs();
        fs.inner.rename(
            self.ino() as _,
            old_name,
            target.inner.write().inode.id as _,
            new_name,
        );
        Ok(())
    }

    fn read_link(&self) -> Result<String> {
        if self.type_() != InodeType::SymLink {
            return_errno!(Errno::EISDIR);
        }
        let file_size = self.size();
        let mut symlink = vec![0u8; file_size];
        let fs = self.fs();
        fs.inner.read(self.ino() as _, 0, symlink.as_mut_slice());
        Ok(String::from_utf8(symlink)?)
    }

    fn write_link(&self, target: &str) -> Result<()> {
        if self.type_() != InodeType::SymLink {
            return_errno!(Errno::EISDIR);
        }
        let file_size = self.size();
        let mut symlink = vec![0u8; file_size];
        let fs = self.fs();
        fs.inner.write(self.ino() as _, 0, target.as_bytes());
        Ok(())
    }

    fn fallocate(&self, mode: FallocMode, offset: usize, len: usize) -> Result<()> {
        unimplemented!()
    }

    fn ioctl(&self, cmd: IoctlCmd, arg: usize) -> Result<i32> {
        Err(Error::new(Errno::EINVAL))
    }

    fn sync_all(&self) -> Result<()> {
        let fs = self.fs();
        fs.inner.flush_all();
        Ok(())
    }

    fn sync_data(&self) -> Result<()> {
        let fs = self.fs();
        fs.inner.flush_all();
        Ok(())
    }

    fn fs(&self) -> Arc<dyn FileSystem> {
        self.fs.upgrade().unwrap()
    }

    fn extension(&self) -> Option<&Extension> {
        None
    }

    fn set_xattr(
        &self,
        name: XattrName,
        value_reader: &mut VmReader,
        flags: XattrSetFlags,
    ) -> Result<()> {
        Ok(())
    }

    fn get_xattr(&self, name: XattrName, value_writer: &mut VmWriter) -> Result<usize> {
        Ok(0)
    }

    fn list_xattr(&self, namespace: XattrNamespace, list_writer: &mut VmWriter) -> Result<usize> {
        Ok(0)
    }

    fn remove_xattr(&self, name: XattrName) -> Result<()> {
        Ok(())
    }
}

/// The Ext4 filesystem.
pub struct Ext4 {
    inner: another_ext4::Ext4,
    self_ref: Weak<Self>,
}

impl Ext4 {
    /// Opens and loads an Ext4 from the `block_device`.
    pub fn open(block_device: Arc<dyn BlockDevice>) -> Result<Arc<Self>> {
        let block_device = Arc::new(BlockDeviceWrapper {
            inner: block_device,
        });
        let ext4 = Arc::new_cyclic(|weak_ref| Self {
            inner: another_ext4::Ext4::load(block_device).unwrap(),
            self_ref: weak_ref.clone(),
        });
        Ok(ext4)
    }

    pub fn root_inode(&self) -> Result<Arc<dyn Inode>> {
        let inode = self.inner.read_inode(ROOT_INO as _);
        Ok(Arc::new(Ext4Inode {
            inner: RwMutex::new(Ext4InodeInner { inode }),
            fs: self.self_ref.clone(),
        }))
    }
}

impl FileSystem for Ext4 {
    fn sync(&self) -> Result<()> {
        self.inner.flush_all();
        Ok(())
    }

    fn root_inode(&self) -> Arc<dyn Inode> {
        self.root_inode().unwrap()
    }

    fn sb(&self) -> SuperBlock {
        let sb = self.inner.read_super_block();

        SuperBlock {
            magic: 0x0,
            bsize: BLOCK_SIZE,
            blocks: sb.block_count() as _,
            bfree: sb.free_blocks_count() as _,
            bavail: sb.free_blocks_count() as _,
            files: sb.inode_count() as _,
            ffree: sb.free_inodes_count() as _,
            fsid: 0, // TODO
            namelen: NAME_MAX,
            frsize: BLOCK_SIZE,
            flags: 0, // TODO
        }
    }

    fn flags(&self) -> FsFlags {
        FsFlags::empty()
    }
}

/// Returns the current time.
pub fn now() -> Duration {
    crate::time::clocks::RealTimeCoarseClock::get().read_time()
}

fn is_block_aligned(offset: usize) -> bool {
    offset % BLOCK_SIZE == 0
}
