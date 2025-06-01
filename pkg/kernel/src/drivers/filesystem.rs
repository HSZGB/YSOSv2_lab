use super::ata::*;
use alloc::boxed::Box;
use alloc::format;
use chrono::DateTime;
use storage::fat16::Fat16;
use storage::mbr::*;
use storage::*;

pub static ROOTFS: spin::Once<Mount> = spin::Once::new();

pub fn get_rootfs() -> &'static Mount {
    ROOTFS.get().unwrap()
}

pub fn init() {
    info!("Opening disk device...");

    let drive = AtaDrive::open(0, 0).expect("Failed to open disk device");

    // only get the first partition
    let part = MbrTable::parse(drive)
        .expect("Failed to parse MBR")
        .partitions()
        .expect("Failed to get partitions")
        .remove(0);

    info!("Mounting filesystem...");

    ROOTFS.call_once(|| Mount::new(Box::new(Fat16::new(part)), "/".into()));

    trace!("Root filesystem: {:#?}", ROOTFS.get().unwrap());

    info!("Initialized Filesystem.");
}

pub fn ls(root_path: &str) {
    let iter = match get_rootfs().read_dir(root_path) {
        Ok(iter) => iter,
        Err(err) => {
            warn!("{:?}", err);
            return;
        }
    };

    // FIXME: format and print the file metadata
    //      - use `for meta in iter` to iterate over the entries
    //      - use `crate::humanized_size_short` for file size
    //      - add '/' to the end of directory names
    //      - format the date as you like
    //      - do not forget to print the table header
    
    println!("{:<15} {:>8} {:<10} {:<19}", "Name", "Size", "Type", "Modified");

    for meta in iter {
        let name = match meta.entry_type {
            FileType::Directory => format!("{}/", meta.name),
            FileType::File => meta.name.clone(),
        };

        let (size, unit) = crate::humanized_size_short(meta.len as u64);
        let size_str = format!("{:.1}{}", size, unit);

        let typ = match meta.entry_type {
            FileType::Directory => "dir",
            FileType::File => "file",
        };

        let modified= meta.modified
            .map(|t| t.format("%Y/%m/%d %H:%M:%S"))
            .unwrap_or(
                DateTime::from_timestamp_millis(0)
                    .unwrap()
                    .format("%Y/%m/%d %H:%M:%S")
            );

        println!("{:<15} {:>8} {:<10} {:<19}", name, size_str, typ, modified);
    }
}
