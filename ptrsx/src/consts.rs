#[cfg(target_os = "linux")]
pub const CHUNK_SIZE: usize = 0x100000;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub const CHUNK_SIZE: usize = 0x4000;

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub const CHUNK_SIZE: usize = 0x1000;

#[cfg(all(target_os = "windows"))]
pub const CHUNK_SIZE: usize = 0x1000;

pub const POINTER_SIZE: usize = core::mem::size_of::<Address>();

pub const MAX_BUF_SIZE: usize = 0x100000;

pub type Address = usize;

#[cfg(target_os = "linux")]
pub const EXE: [[u8; 4]; 1] = [[0x7f, b'E', b'L', b'F']];

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub const EXE: [[u8; 4]; 2] = [[0xCA, 0xFE, 0xBA, 0xBE], [0xCF, 0xFA, 0xED, 0xFE]];

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub const EXE: [&str; 2] = ["exe", "dll"];

pub const MODEL1: [u8; 8] = 1_u64.to_le_bytes();
pub const MODEL2: [u8; 8] = 2_u64.to_le_bytes();
