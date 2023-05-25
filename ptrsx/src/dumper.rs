use std::{cmp::Ordering, io, mem};

use vmmap::{Pid, Process, ProcessInfo, VirtualMemoryRead, VirtualQuery};

use super::{
    check::check_region,
    consts::{Address, CHUNK_SIZE, POINTER_SIZE},
    error::Error,
    map::{encode_map_to_writer, Page},
};

pub struct PtrsXDumper {
    pub proc: Process,
}

impl PtrsXDumper {
    pub fn init(pid: Pid) -> Result<PtrsXDumper, vmmap::Error> {
        let proc = Process::open(pid)?;
        Ok(Self { proc })
    }

    pub fn create_pointer_map_helper<W>(&self, out: &mut W) -> Result<(), Error>
    where
        W: io::Write,
    {
        let region = self.proc.get_maps().filter(check_region).collect::<Vec<_>>();

        let scan_region = region.iter().map(|m| (m.start(), m.size())).collect::<Vec<_>>();

        let map = region
            .into_iter()
            .filter_map(|m| {
                Some(Page {
                    start: m.start(),
                    end: m.end(),
                    path: m.path().map(|p| p.to_path_buf())?,
                })
            })
            .collect::<Vec<_>>();

        if map.is_empty() {
            return Err("InvalidBaseModule".into());
        }

        let map = merge_bases(map);
        encode_map_to_writer(map, out)?;

        Ok(create_pointer_map(&self.proc, &scan_region, out)?)
    }
}

fn create_pointer_map<P, W>(proc: &P, region: &[(Address, Address)], out: &mut W) -> io::Result<()>
where
    P: VirtualMemoryRead,
    W: io::Write,
{
    let mut buf = [0; CHUNK_SIZE];

    for &(start, size) in region {
        for off in (0..size).step_by(CHUNK_SIZE) {
            let Ok (size) = proc.read_at((start + off) as _, buf.as_mut_slice()) else {
                break;
            };
            for (k, buf) in buf[..size].windows(POINTER_SIZE).enumerate() {
                let addr = start + off + k;
                let out_addr = unsafe { mem::transmute::<[u8; POINTER_SIZE], Address>(*(buf.as_ptr() as *const _)) };
                if region
                    .binary_search_by(|&(a, s)| {
                        if out_addr >= a && out_addr < a + s {
                            Ordering::Equal
                        } else {
                            a.cmp(&out_addr)
                        }
                    })
                    .is_ok()
                {
                    // TODO big_endian, 32 bit, [u64; 2], [u8; 16] , [u32; 2], [u8; 8] ...
                    out.write_all(&unsafe {
                        mem::transmute::<[Address; 2], [u8; POINTER_SIZE * 2]>([addr, out_addr])
                    })?;
                }
            }
        }
    }

    Ok(())
}

#[inline]
pub fn merge_bases(mut bases: Vec<Page>) -> Vec<Page> {
    let mut aom = Vec::new();
    let mut current = core::mem::take(&mut bases[0]);
    for map in bases.into_iter().skip(1) {
        if map.path == current.path {
            current.end = map.end;
        } else {
            aom.push(current);
            current = map;
        }
    }
    aom.push(current);
    aom
}
