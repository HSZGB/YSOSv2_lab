use core::f32::consts::FRAC_1_PI;

use super::*;

impl Fat16Impl {
    pub fn new(inner: impl BlockDevice<Block512>) -> Self {
        let mut block = Block::default();
        let block_size = Block512::size();

        inner.read_block(0, &mut block).unwrap();
        let bpb = Fat16Bpb::new(block.as_ref()).unwrap();

        trace!("Loading Fat16 Volume: {:#?}", bpb);

        // HINT: FirstDataSector = BPB_ResvdSecCnt + (BPB_NumFATs * FATSz) + RootDirSectors;
        let fat_start = bpb.reserved_sector_count() as usize;
        let root_dir_size = (bpb.root_entries_count() as usize * DirEntry::LEN).div_ceil(block_size); /* FIXME: get the size of root dir from bpb */
        let first_root_dir_sector = fat_start + (bpb.fat_count() as usize * bpb.sectors_per_fat() as usize); /* FIXME: calculate the first root dir sector */
        let first_data_sector = first_root_dir_sector + root_dir_size;

        Self {
            bpb,
            inner: Box::new(inner),
            fat_start,
            first_data_sector,
            first_root_dir_sector,
        }
    }

    pub fn cluster_to_sector(&self, cluster: &Cluster) -> usize {
        match *cluster {
            Cluster::ROOT_DIR => self.first_root_dir_sector,
            Cluster(c) => {
                // FIXME: calculate the first sector of the cluster
                // HINT: FirstSectorofCluster = ((N – 2) * BPB_SecPerClus) + FirstDataSector;
                let first_sector_of_cluster = ((c as usize - 2) * self.bpb.sectors_per_cluster() as usize) + self.first_data_sector;
                first_sector_of_cluster
            }
        }
    }

    // FIXME: YOU NEED TO IMPLEMENT THE FILE SYSTEM OPERATIONS HERE
    //      - read the FAT and get next cluster
    //      - traverse the cluster chain and read the data
    //      - parse the path
    //      - open the root directory
    //      - ...
    //      - finally, implement the FileSystem trait for Fat16 with `self.handle`

    /// look for next cluster in FAT
    pub fn next_cluster(&self, cluster: &Cluster) -> FsResult<Cluster> {
        // 1. 计算 FAT 表项在 FAT 表中的字节偏移
        let fat_offset = (cluster.0 * 2) as usize;

        // 2. 计算该偏移对应的扇区
        let block_size = Block512::size();
        let fat_sector = self.fat_start + (fat_offset / block_size);
        let offset = fat_offset % block_size; // 扇区内的偏移
        
        // 3. 读取包含该 FAT 表项的扇区
        let mut block = Block::default();
        self.inner.read_block(fat_sector, &mut block).unwrap();

        let fat_entry = u16::from_le_bytes(block[offset..=offset + 1].try_into().unwrap_or([0; 2]));

        match fat_entry {
            0xFFF7 => Err(FsError::BadCluster),         // Bad cluster
            0xFFF8..=0xFFFF => Err(FsError::EndOfFile), // There is no next cluster
            f => Ok(Cluster(f as u32)),                 // Seems legit
        }
    }

    fn find_entry_in_sector(&self, match_name: &ShortFileName, sector: usize) -> FsResult<DirEntry> {
        let mut block = Block::default();
        self.inner.read_block(sector, &mut block).unwrap();

        for i in (0..BLOCK_SIZE).step_by(DirEntry::LEN) {
            let entry = DirEntry::parse(&block[i..i + DirEntry::LEN]).map_err(|_| FsError::InvalidOperation)?;
            if entry.filename.matches(match_name) {
                return Ok(entry);
            }
        }

        Err(FsError::NotInSector)
    }

    // 目录同样是一个由目录条目构成的数组
    fn find_directory_entry(&self, dir: &Directory, name: &str) -> FsResult<DirEntry> {
        let match_name = ShortFileName::parse(name)?;

        // 根目录（FAT16）：不是用簇链管理的，而是一块固定大小的特殊区域，不需要“找下一个簇”。
        // 普通目录：内容存放在一个或多个簇，这些簇通过FAT表串起来，需要沿着簇链一个个遍历。

        let mut current_cluster = Some(dir.cluster);
        let mut dir_sector_num = self.cluster_to_sector(&dir.cluster);

        let dir_size = match dir.cluster {
            Cluster::ROOT_DIR => self.first_data_sector - self.first_root_dir_sector,
            _ => self.bpb.sectors_per_cluster() as usize,
        };

        while let Some(cluster) = current_cluster {
            for sector in dir_sector_num..dir_sector_num + dir_size {
                match self.find_entry_in_sector(&match_name, sector) {
                    Err(FsError::NotInSector) => continue,
                    x => return x,
                }
            }
            current_cluster = if cluster != Cluster::ROOT_DIR {
                match self.next_cluster(&cluster) {
                    Ok(n) => {
                        dir_sector_num = self.cluster_to_sector(&n);
                        Some(n)
                    }
                    _ => None,
                }
            } else {
                None
            }
        }

        Err(FsError::FileNotFound)
    }

    // 对于文件，返回的是它所在的目录
    // 对于目录，返回该目录
    fn get_parent_dir(&self, path: &str) -> FsResult<Directory> {
        let mut path = path.split(PATH_SEPARATOR);
        let mut current = Directory::root();

        while let Some(dir) = path.next() {
            if dir.is_empty() {
                continue;
            }

            let entry = self.find_directory_entry(&current, dir)?;

            if entry.is_directory() {
                current = Directory::from_entry(entry);
            } else if path.next().is_some() { // 如果当前走到了文件，但是还有后续路径，那么就报错
                return Err(FsError::NotADirectory);
            } else {
                break;
            }
        }

        Ok(current)
    }

    pub fn iterate_dir<F>(&self, dir: &directory::Directory, mut func: F) -> FsResult<()>
    where
        F: FnMut(&DirEntry),
    {
        if let Some(entry) = &dir.entry {
            trace!("Iterating directory: {}", entry.filename());
        }

        let mut current_cluster = Some(dir.cluster);
        let mut dir_sector_num = self.cluster_to_sector(&dir.cluster);
        let dir_size = match dir.cluster {
            Cluster::ROOT_DIR => self.first_data_sector - self.first_root_dir_sector,
            _ => self.bpb.sectors_per_cluster() as usize,
        };
        trace!("Directory size: {}", dir_size);

        let mut block = Block::default();
        let block_size = Block512::size();
        while let Some(cluster) = current_cluster {
            for sector in dir_sector_num..dir_sector_num + dir_size {
                self.inner.read_block(sector, &mut block).unwrap();
                for entry in 0..block_size / DirEntry::LEN {
                    let start = entry * DirEntry::LEN;
                    let end = (entry + 1) * DirEntry::LEN;

                    let dir_entry = DirEntry::parse(&block[start..end])?;

                    if dir_entry.is_eod() {
                        return Ok(());
                    } else if dir_entry.is_valid() && !dir_entry.is_long_name() {
                        func(&dir_entry);
                    }
                }
            }
            current_cluster = if cluster != Cluster::ROOT_DIR {
                match self.next_cluster(&cluster) {
                    Ok(n) => {
                        dir_sector_num = self.cluster_to_sector(&n);
                        Some(n)
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        Ok(())
    }

    pub fn get_dir_entry(&self, path: &str) -> FsResult<DirEntry> {
        let parent = self.get_parent_dir(path)?;
        let name = path.rsplit(PATH_SEPARATOR).next().unwrap_or("");

        self.find_directory_entry(&parent, name)
    }

}

impl FileSystem for Fat16 {
    fn read_dir(&self, path: &str) -> FsResult<Box<dyn Iterator<Item = Metadata> + Send>> {
        // FIXME: read dir and return an iterator for all entries
        let dir = self.handle.get_parent_dir(path)?;
        let mut entries = Vec::new();

        self.handle.iterate_dir(&dir, |entry| {
            entries.push(entry.as_meta());
        })?;

        Ok(Box::new(entries.into_iter()))
    }

    fn open_file(&self, path: &str) -> FsResult<FileHandle> {
        // FIXME: open file and return a file handle
        let entry = self.handle.get_dir_entry(path)?;

        if entry.is_directory() {
            return Err(FsError::NotAFile);
        }

        let handle = self.handle.clone();
        let meta = entry.as_meta();
        let file = Box::new(File::new(handle, entry));

        let file_handle = FileHandle::new(meta, file);

        Ok(file_handle)
    }

    fn metadata(&self, path: &str) -> FsResult<Metadata> {
        // FIXME: read metadata of the file / dir
        todo!()
    }

    fn exists(&self, path: &str) -> FsResult<bool> {
        // FIXME: check if the file / dir exists
        todo!()
    }
}
