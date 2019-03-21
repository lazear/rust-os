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
    let build = Command::new("cargo")
        .current_dir("kernel")
        .args(&["xbuild", "--target", "target.json", "--release"])
        .spawn()?
        .wait()?
        .success();
    if !build {
        panic!("Error executing cargo xbuild");
    }

    let ld = Command::new("ld")
        .args(&[
            "--gc-sections",
            "-z",
            "max-page-size=0x1000",
            "-o",
            "./build/kernel.elf",
            "-T",
            "linker.ld",
            "./target/target/release/librust_os.a",
        ])
        .spawn()?
        .wait()?;

    let nasm = Command::new("nasm")
        .current_dir("bootloader")
        .args(&["-f", "bin", "stage1.asm", "-o", "../build/bootstrap.bin"])
        .spawn()?
        .wait()?
        .success();
    if !nasm {
        panic!("Error executing assembler commands");
    }

    println!("Copying files to disk image");

    let mut handle = create_block("./build/disk.img", 0x1000)?;
    let mut bootloader = File::open("./build/bootstrap.bin")?;
    let mut kernel = File::open("./build/kernel.elf")?;

    let data = kernel.metadata()?.len() as usize;

    copy_to_file(&mut handle, &mut bootloader, 0)?;
    assert_eq!(data, copy_to_file(&mut handle, &mut kernel, 0x400)?);

    Ok(())
}
