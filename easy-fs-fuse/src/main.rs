use clap::{App, Arg};
use easy_fs::{BlockDevice, EasyFileSystem};
use std::{
    fs::{read_dir, File, OpenOptions},
    io::{Read, Seek, SeekFrom},
    sync::{Arc, Mutex},
};

const BLOCK_SZ: usize = 512;

struct BlockFile(Mutex<File>);

impl BlockDevice for BlockFile {
    fn read_block(&self, block_num: usize, buf: &mut [u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_num * BLOCK_SZ) as u64))
            .unwrap();
        file.read_exact(buf).unwrap();
    }

    fn write_block(&self, block_num: usize, buf: &[u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_num * BLOCK_SZ) as u64))
            .unwrap();
        file.write_all(buf).unwrap();
    }
}

fn main() {
    easy_fs_pack().expect("Failed to pack filesystem");
}

fn easy_fs_pack() -> std::io::Result<()> {
    let matches = App::new("EasyFS packer")
        .version("0.1")
        .author("Your Name")
        .about("Pack EasyFS filesystem")
        .arg(
            Arg::with_name("source")
                .short("s")
                .long("source")
                .takes_value(true)
                .help("Executable source dir(with backslash)"),
        )
        .arg(
            Arg::with_name("target")
                .short("t")
                .long("target")
                .takes_value(true)
                .help("Executable target dir(with backslash)"),
        )
        .get_matches();
    let src_path = matches.value_of("source").unwrap();
    let target_path = matches.value_of("target").unwrap();

    println!("src_path: {}\ntarget_path: {}", src_path, target_path);
    let block_file = Arc::new(BlockFile(Mutex::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(format!("{}{}", target_path, "easy_fs.img"))?;
        // 16MiB, at most 4095 files
        f.set_len(16 * 2048 * 512)?;
        f
    })));

    let efs = EasyFileSystem::create(block_file, 16 * 2048, 1);
    let root_inode = Arc::new(EasyFileSystem::root_inode(&efs));
    let apps: Vec<_> = read_dir(src_path)?
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry?.file_name().into_string()?;
            name_with_ext.drain(name_with_ext.find(".").unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    for app in apps {
        let mut host_file = File::open(format!("{}{}", target_path, app))?;
        let mut all_data: Vec<u8> = Vec::new();
        host_file.read_to_end(&mut all_data)?;
        let inode = root_inode.create(app.as_str())?;
        inode.write_at(0, &all_data)?;
    }
    for app in root_inode.ls() {
        println!("{}", app);
    }

    Ok(())
}
