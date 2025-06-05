//! File
//!
//! reference: <https://wiki.osdev.org/FAT#Directories_on_FAT12.2F16.2F32>

use super::*;

#[derive(Debug, Clone)]
pub struct File {
    /// The current offset in the file
    offset: usize,
    /// The current cluster of this file
    current_cluster: Cluster,
    /// DirEntry of this file
    entry: DirEntry,
    /// The file system handle that contains this file
    handle: Fat16Handle,
}

impl File {
    pub fn new(handle: Fat16Handle, entry: DirEntry) -> Self {
        Self {
            offset: 0,
            current_cluster: entry.cluster,
            entry,
            handle,
        }
    }

    pub fn length(&self) -> usize {
        self.entry.size as usize
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> FsResult<usize> {

        // todo!("Reading from file is not implemented yet");
        // FIXME: read file content from disk
        //      CAUTION: file length / buffer size / offset
        //
        //      - `self.offset` is the current offset in the file in bytes
        //      - use `self.handle` to read the blocks
        //      - use `self.entry` to get the file's cluster
        //      - use `self.handle.cluster_to_sector` to convert cluster to sector
        //      - update `self.offset` after reading
        //      - update `self.cluster` with FAT if necessary
        
        // 检查是否已经到达文件末尾
        if self.offset >= self.length() {
            return Ok(0);
        }

        // 计算实际可读取的字节数
        let remaining_bytes = self.length() - self.offset;
        let bytes_to_read = buf.len().min(remaining_bytes);
        
        if bytes_to_read == 0 {
            return Ok(0);
        }

        let mut bytes_read = 0;

        while bytes_read < bytes_to_read {
            // 计算当前cluster内的偏移
            let cluster_size = self.handle.bpb.sectors_per_cluster() as usize * self.handle.bpb.bytes_per_sector() as usize;
            let offset_in_cluster = self.offset % cluster_size;
            
            // debug
            let offset_in_sector = offset_in_cluster % self.handle.bpb.bytes_per_sector() as usize;
            let bytes_left_in_sector = BLOCK_SIZE - offset_in_sector;
            let bytes_to_read_from_sector = (bytes_to_read - bytes_read).min(bytes_left_in_sector).min(self.length() - self.offset);

            // 将cluster转换为sector
            // 这个current_cluster是文件的起始cluster，还要算上偏移的
            let sector = self.handle.cluster_to_sector(&self.current_cluster);
            let sector = sector + (offset_in_cluster / self.handle.bpb.bytes_per_sector() as usize);
            
            // 读取数据
            let mut sector_buffer = Block::default();
            self.handle.inner.read_block(sector, &mut sector_buffer)?;
            
            // 复制需要的数据到目标buffer
            let src_start = offset_in_sector;
            let src_end = src_start + bytes_to_read_from_sector;
            let dst_start = bytes_read;
            let dst_end = dst_start + bytes_to_read_from_sector;
            
            buf[dst_start..dst_end].copy_from_slice(&sector_buffer[src_start..src_end]);
            
            // 更新偏移量和已读字节数
            self.offset += bytes_to_read_from_sector;
            bytes_read += bytes_to_read_from_sector;
            
            // 如果读完了当前cluster且还需要继续读取，则移动到下一个cluster
            if self.offset % cluster_size == 0 {
                // 通过FAT表获取下一个cluster
                // 检查是否到达文件末尾
                if let Ok(next_cluster) = self.handle.next_cluster(&self.current_cluster) {
                    self.current_cluster = next_cluster;
                } else {
                    break;
                }
            }
        }

        Ok(bytes_read)
    }
}

// NOTE: `Seek` trait is not required for this lab
impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> FsResult<usize> {
        unimplemented!()
    }
}

// NOTE: `Write` trait is not required for this lab
impl Write for File {
    fn write(&mut self, _buf: &[u8]) -> FsResult<usize> {
        unimplemented!()
    }

    fn flush(&mut self) -> FsResult {
        unimplemented!()
    }
}
