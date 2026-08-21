//! Computes the [`HeaderV0::hash_digest`] value with SHA-1.
use std::{fs::File, process::ExitCode};

use abootimg_oxide::HeaderV0;

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let Ok([hash_type, paths @ ..]) = TryInto::<[_; 6]>::try_into(args) else {
        eprintln!("Usage: compute-hash-digest <--sha1|--sha256> <kernel> <ramdisk> <second bootloader> <recovery dtbo> <dtb>\n\nEmpty paths and errors are ignored.");
        return ExitCode::FAILURE;
    };

    let files = paths.map(|path| {
        if path.is_empty() {
            None
        } else {
            File::open(path).ok()
        }
    });

    let mut files = files.each_ref().map(|opt| opt.as_ref());
    let files = files.each_mut().map(|opt| opt.as_mut());

    let hash_digest = match hash_type.as_str() {
        "--sha1" => hash_files::<sha1::Sha1>(files),
        "--sha256" => hash_files::<sha2::Sha256>(files),
        _ => {
            eprintln!("Unknown hash type: {hash_type}.\n\nUsage: compute-hash-digest <--sha1|--sha256> <kernel> <ramdisk> <second bootloader> <recovery dtbo> <dtb>\n\nEmpty paths and errors are ignored.");
            return ExitCode::FAILURE;
        }
    };

    println!("The hash digest is {}", hex_fmt::HexFmt(hash_digest));
    ExitCode::SUCCESS
}

fn hash_files<D: digest::Digest>(files: [Option<&mut &File>; 5]) -> [u8; 32] {
    let [kernel_f, ramdisk_f, second_bootloader_f, recovery_dtbo_f, dtb_f] = files;

    HeaderV0::compute_hash_digest::<_, D>(
        kernel_f,
        ramdisk_f,
        second_bootloader_f,
        recovery_dtbo_f,
        dtb_f,
    )
    .unwrap()
}
