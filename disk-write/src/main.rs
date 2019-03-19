use std::fs::File;
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};
use std::process::{Command, Stdio};

fn create_block(output: &str, blocks: usize) -> io::Result<BufWriter<File>> {
    let mut handle = BufWriter::new(File::create(output)?);
    let buffer = [0u8; 1024];
    for _ in 0..blocks {
        handle.write_all(&buffer)?;
    }
    Ok(handle)
}

fn copy_to_file<W: Write + Seek, R: Read>(
    output: &mut W,
    input: &mut R,
    offset: u64,
) -> io::Result<usize> {
    output.seek(SeekFrom::Start(offset))?;
    let mut written = 0;
    let mut buffer = [0u8; 1024];
    loop {
        let n = match input.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => return Err(e),
        };
        output.write_all(&mut buffer[..n])?;
        written += n;
    }
    Ok(written)
}

fn main() -> io::Result<()> {
    let mut build = Command::new("cargo")
        .current_dir("../kernel")
        .args(&["xbuild", "--target", "target.json", "--release"])
        .spawn()?;
    if !build.wait()?.success() {
        panic!("Error executing cargo xbuild");
    }

    let mut handle = create_block("../disk.img", 0x1000)?;
    let mut bootloader = File::open("../bootloader/bootstrap")?;
    let mut kernel = File::open("../kernel/target/target/release/rust-os")?;

    let data = kernel.metadata()?.len() as usize;

    copy_to_file(&mut handle, &mut bootloader, 0)?;
    assert_eq!(data, copy_to_file(&mut handle, &mut kernel, 0x400)?);

    let mut qemu = Command::new("C:\\Program Files\\qemu\\qemu-system-x86_64")
        .args(&["disk.img", "--monitor", "stdio"])
        .current_dir("../")
        .spawn()?;


    Ok(())
}
