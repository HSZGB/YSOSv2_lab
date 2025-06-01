//! Directory Entry
//!
//! reference: <https://wiki.osdev.org/FAT#Directories_on_FAT12.2F16.2F32>

use crate::*;
use bitflags::bitflags;
use chrono::LocalResult::Single;
use chrono::{DateTime, TimeZone, Utc};
use core::fmt::{Debug, Display};
use core::ops::*;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DirEntry {
    pub filename: ShortFileName,
    pub modified_time: FsTime,
    pub created_time: FsTime,
    pub accessed_time: FsTime,
    pub cluster: Cluster,
    pub attributes: Attributes,
    pub size: u32,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Cluster(pub u32);

bitflags! {
    /// File Attributes
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Attributes: u8 {
        const READ_ONLY = 0x01;
        const HIDDEN    = 0x02;
        const SYSTEM    = 0x04;
        const VOLUME_ID = 0x08;
        const DIRECTORY = 0x10;
        const ARCHIVE   = 0x20;
        const LFN       = 0x0f; // Long File Name, Not Implemented
    }
}

impl DirEntry {
    pub const LEN: usize = 0x20;

    pub fn is_valid(&self) -> bool {
        !self.filename.is_eod() && !self.filename.is_unused()
    }

    pub fn is_long_name(&self) -> bool {
        self.attributes.contains(Attributes::LFN)
    }

    pub fn is_directory(&self) -> bool {
        self.attributes.contains(Attributes::DIRECTORY)
    }

    pub fn is_eod(&self) -> bool {
        self.filename.is_eod()
    }

    pub fn filename(&self) -> String {
        // NOTE: ignore the long file name in FAT16 for lab
        if self.is_valid() && !self.is_long_name() {
            format!("{}", self.filename)
        } else {
            String::from("unknown")
        }
    }

    /// For Standard 8.3 format
    ///
    /// reference: https://osdev.org/FAT#Standard_8.3_format
    pub fn parse(data: &[u8]) -> FsResult<DirEntry> {
        let filename = ShortFileName::new(&data[..11]);

        // FIXME: parse the rest of the fields
        //      - ensure you can pass the test
        //      - you may need `prase_datetime` function

        let modified_time = prase_datetime(u32::from_le_bytes(data[22..26].try_into().unwrap()));
        let created_time = prase_datetime(u32::from_le_bytes(data[14..18].try_into().unwrap()));
        let accessed_time = prase_datetime(u32::from_le_bytes([0, 0, data[18], data[19]]));

        let cluster = ((data[27] as u32) << 8)
            | (data[26] as u32)
            | ((data[21] as u32) << 24)
            | ((data[20] as u32) << 16);

        let attributes = Attributes::from_bits_truncate(data[11]);

        let size = u32::from_le_bytes(data[28..32].try_into().unwrap());

        Ok(DirEntry {
            filename,
            modified_time,
            created_time,
            accessed_time,
            cluster: Cluster(cluster),
            attributes,
            size,
        })
    }

    pub fn as_meta(&self) -> Metadata {
        self.into()
    }
}

fn prase_datetime(time: u32) -> FsTime {
    // FIXME: parse the year, month, day, hour, min, sec from time
    let year = ((time >> 25) + 1980) as i32;
    let month = (time >> 21) & 0x0f;
    let day = (time >> 16) & 0x1f;
    let hour = (time >> 11) & 0x1f;
    let min = (time >> 5) & 0x3f;
    let sec = (time & 0x1f) * 2;

    if let Single(time) = Utc.with_ymd_and_hms(year, month, day, hour, min, sec) {
        time
    } else {
        DateTime::from_timestamp_millis(0).unwrap()
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct ShortFileName {
    pub name: [u8; 8],
    pub ext: [u8; 3],
}

impl ShortFileName {
    pub fn new(buf: &[u8]) -> Self {
        Self {
            name: buf[..8].try_into().unwrap(),
            ext: buf[8..11].try_into().unwrap(),
        }
    }

    pub fn basename(&self) -> &str {
        core::str::from_utf8(&self.name).unwrap()
    }

    pub fn extension(&self) -> &str {
        core::str::from_utf8(&self.ext).unwrap()
    }

    pub fn is_eod(&self) -> bool {
        self.name[0] == 0x00 && self.ext[0] == 0x00
    }

    pub fn is_unused(&self) -> bool {
        self.name[0] == 0xE5
    }

    pub fn matches(&self, sfn: &ShortFileName) -> bool {
        self.name == sfn.name && self.ext == sfn.ext
    }

    /// Parse a short file name from a string
    pub fn parse(name: &str) -> FsResult<ShortFileName> {
        // FIXME: implement the parse function
        //      use `FilenameError` and into `FsError`
        //      use different error types for following conditions:
        //
        //      - use 0x20 ' ' for right padding
        //      - check if the filename is empty
        //      - check if the name & ext are too long
        //      - period `.` means the start of the file extension
        //      - check if the period is misplaced (after 8 characters)
        //      - check if the filename contains invalid characters:
        //        [0x00..=0x1F, 0x20, 0x22, 0x2A, 0x2B, 0x2C, 0x2F, 0x3A,
        //        0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x5B, 0x5C, 0x5D, 0x7C]
        let mut sfn = ShortFileName {
            name: [0x20; 8],
            ext: [0x20; 3],
        };
        let mut idx = 0;
        let mut seen_dot = false;
        for ch in name.bytes() {
            match ch {
                // Microsoft say these are the invalid characters
                0x00..=0x1F
                | 0x20
                | 0x22
                | 0x2A
                | 0x2B
                | 0x2C
                | 0x2F
                | 0x3A
                | 0x3B
                | 0x3C
                | 0x3D
                | 0x3E
                | 0x3F
                | 0x5B
                | 0x5C
                | 0x5D
                | 0x7C => {
                    return Err(FilenameError::InvalidCharacter.into());
                }
                // Denotes the start of the file extension
                b'.' => {
                    if (1..=8).contains(&idx) {
                        seen_dot = true;
                        idx = 8;
                    } else {
                        return Err(FilenameError::MisplacedPeriod.into());
                    }
                }
                _ => {
                    let ch = ch.to_ascii_uppercase();
                    // trace!("Char: '{}', at: {}", ch as char, idx);
                    if seen_dot {
                        if (8..11).contains(&idx) {
                            sfn.ext[idx - 8] = ch;
                        } else {
                            return Err(FilenameError::NameTooLong.into());
                        }
                    } else if idx < 8 {
                        sfn.name[idx] = ch;
                    } else {
                        return Err(FilenameError::NameTooLong.into());
                    }
                    idx += 1;
                }
            }
        }
        if idx == 0 {
            return Err(FilenameError::FilenameEmpty.into());
        }
        Ok(sfn)
    }
}

impl Debug for ShortFileName {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for ShortFileName {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if self.ext[0] == 0x20 {
            write!(f, "{}", self.basename().trim_end())
        } else {
            write!(
                f,
                "{}.{}",
                self.basename().trim_end(),
                self.extension().trim_end()
            )
        }
    }
}

impl Cluster {
    /// Magic value indicating an invalid cluster value.
    pub const INVALID: Cluster = Cluster(0xFFFF_FFF6);
    /// Magic value indicating a bad cluster.
    pub const BAD: Cluster = Cluster(0xFFFF_FFF7);
    /// Magic value indicating a empty cluster.
    pub const EMPTY: Cluster = Cluster(0x0000_0000);
    /// Magic value indicating the cluster holding the root directory
    /// (which doesn't have a number in Fat16 as there's a reserved region).
    pub const ROOT_DIR: Cluster = Cluster(0xFFFF_FFFC);
    /// Magic value indicating that the cluster is allocated and is the final cluster for the file
    pub const END_OF_FILE: Cluster = Cluster(0xFFFF_FFFF);
}

impl Add<u32> for Cluster {
    type Output = Cluster;
    fn add(self, rhs: u32) -> Cluster {
        Cluster(self.0 + rhs)
    }
}

impl AddAssign<u32> for Cluster {
    fn add_assign(&mut self, rhs: u32) {
        self.0 += rhs;
    }
}

impl Add<Cluster> for Cluster {
    type Output = Cluster;
    fn add(self, rhs: Cluster) -> Cluster {
        Cluster(self.0 + rhs.0)
    }
}

impl AddAssign<Cluster> for Cluster {
    fn add_assign(&mut self, rhs: Cluster) {
        self.0 += rhs.0;
    }
}

impl From<&DirEntry> for Metadata {
    fn from(entry: &DirEntry) -> Metadata {
        Metadata {
            entry_type: if entry.is_directory() {
                FileType::Directory
            } else {
                FileType::File
            },
            name: entry.filename(),
            len: entry.size as usize,
            created: Some(entry.created_time),
            accessed: Some(entry.accessed_time),
            modified: Some(entry.modified_time),
        }
    }
}

impl Display for Cluster {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "0x{:08X}", self.0)
    }
}

impl Debug for Cluster {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "0x{:08X}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dir_entry() {
        let data = hex_literal::hex!(
            "4b 45 52 4e 45 4c 20 20 45 4c 46 20 00 00 0f be
             d0 50 d0 50 00 00 0f be d0 50 02 00 f0 e4 0e 00"
        );

        let res = DirEntry::parse(&data).unwrap();

        assert_eq!(&res.filename.name, b"KERNEL  ");
        assert_eq!(&res.filename.ext, b"ELF");
        assert_eq!(res.attributes, Attributes::ARCHIVE);
        assert_eq!(res.cluster, Cluster(2));
        assert_eq!(res.size, 0xee4f0);
        assert_eq!(
            res.created_time,
            Utc.with_ymd_and_hms(2020, 6, 16, 23, 48, 30).unwrap()
        );
        assert_eq!(
            res.modified_time,
            Utc.with_ymd_and_hms(2020, 6, 16, 23, 48, 30).unwrap()
        );
        assert_eq!(
            res.accessed_time,
            Utc.with_ymd_and_hms(2020, 6, 16, 0, 0, 0).unwrap()
        );

        println!("{:#?}", res);
    }
}
