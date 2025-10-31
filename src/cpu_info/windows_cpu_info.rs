use windows::{
    Win32::System::{
        Registry::{
            HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_VALUE_TYPE, RegOpenKeyExW, RegQueryValueExW,
        },
        SystemInformation::{
            GetLogicalProcessorInformationEx, GetTickCount64, LOGICAL_PROCESSOR_RELATIONSHIP,
            RelationProcessorCore, SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
        },
    },
    core::PCWSTR,
};

use crate::cpu_info::{CpuData, CpuInfo};

pub struct WindowsCpuInfo(CpuData);
impl CpuInfo for WindowsCpuInfo {
    fn new() -> Self {
        let mut results = CpuData::default();
        results.arch = std::env::consts::ARCH.to_string();

        //Vendor
        let (_, b, c, d) = cpuid(0, 0);
        let mut vendor = Vec::new();
        vendor.extend_from_slice(&b.to_le_bytes());
        vendor.extend_from_slice(&d.to_le_bytes());
        vendor.extend_from_slice(&c.to_le_bytes());
        results.vendor = String::from_utf8_lossy(&vendor).trim().to_string();

        //Brand
        let max_ext = cpuid(0x80000000, 0).0;
        if max_ext >= 0x80000004 {
            for leaf in 0x80000002..=0x80000004 {
                let (ea, eb, ec, ed) = cpuid(leaf, 0);
                for r in [ea, eb, ec, ed] {
                    results
                        .brand
                        .push_str(&String::from_utf8_lossy(&r.to_le_bytes()));
                }
            }
            results.brand = results.brand.trim_matches(char::from(0)).trim().to_string();
        }

        let (eax, _, ecx, edx) = cpuid(1, 0);

        //Family/Model/Stepping
        results.stepping = (eax & 0xF) as u8;
        let model = ((eax >> 4) & 0xF) as u8;
        let family = ((eax >> 8) & 0xF) as u8;
        let ext_model = ((eax >> 16) & 0xF) as u8;
        let ext_family = ((eax >> 20) & 0xFF) as u8;
        results.family = if family == 0xF {
            family + ext_family
        } else {
            family
        };
        results.model = if family == 0x6 || family == 0xF {
            (ext_model << 4) + model
        } else {
            model
        };

        //Flags
        if (ecx & (1 << 23)) != 0 {
            results.flags.push("POP_CNT".into());
        }
        if (edx & (1 << 25)) != 0 {
            results.flags.push("SSE".into());
        }
        if (edx & (1 << 26)) != 0 {
            results.flags.push("SSE2".into());
        }
        if (ecx & (1 << 0)) != 0 {
            results.flags.push("SSE3".into());
        }
        if (ecx & (1 << 9)) != 0 {
            results.flags.push("SSSE3".into());
        }
        if (ecx & (1 << 19)) != 0 {
            results.flags.push("SSE4.1".into());
        }
        if (ecx & (1 << 20)) != 0 {
            results.flags.push("SSE4.2".into());
        }
        if (ecx & (1 << 25)) != 0 {
            results.flags.push("AES".into());
        }
        if (ecx & (1 << 28)) != 0 {
            results.flags.push("AVX".into());
        }

        let (_, ebx, _, _) = cpuid(7, 0);
        if (ebx & (1 << 5)) != 0 {
            results.flags.push("AVX2".into());
        }
        if (ebx & (1 << 8)) != 0 {
            results.flags.push("BMI2".into());
        }
        if (ebx & (1 << 16)) != 0 {
            results.flags.push("AVX512F".into());
        }
        if (ebx & (1 << 30)) != 0 {
            results.flags.push("AVX512BW".into());
        }
        if (ebx & (1 << 31)) != 0 {
            results.flags.push("AVX512V1".into());
        }

        //Arch
        (
            results.cores,
            results.threads,
            results.l1_cache,
            results.l2_cache,
            results.l3_cache,
        ) = topo_and_caches();

        //Clock
        (results.clock_speed, results.clock_speed_turbo, _, _) = cpuid(0x16, 0);
        if results.clock_speed == 0
            && let Some(base_clock) = read_registry_mhz()
        {
            results.clock_speed = base_clock
        }

        //Microcode
        results.microcode = read_microcode().unwrap_or_default();

        Self(results)
    }

    fn static_data(&self) -> &CpuData {
        &self.0
    }

    fn uptime(&self) -> f64 {
        unsafe { GetTickCount64() as f64 / 1000.0 }
    }
}

#[inline]
fn cpuid(eax: u32, ecx: u32) -> (u32, u32, u32, u32) {
    if cfg!(target_arch = "x86_64") {
        unsafe {
            let r = core::arch::x86_64::__cpuid_count(eax, ecx);
            (r.eax, r.ebx, r.ecx, r.edx)
        }
    } else {
        (0, 0, 0, 0)
    }
}

fn topo_and_caches() -> (usize, usize, usize, usize, usize) {
    let mut needed: u32 = 0;
    _ = unsafe {
        GetLogicalProcessorInformationEx(LOGICAL_PROCESSOR_RELATIONSHIP(0), None, &mut needed)
    };
    let mut buf = vec![0u8; needed as usize];

    let ok = unsafe {
        GetLogicalProcessorInformationEx(
            LOGICAL_PROCESSOR_RELATIONSHIP(0),
            Some(buf.as_mut_ptr() as *mut _),
            &mut needed,
        )
    };
    if ok.is_err() {
        return (0, 0, 0, 0, 0);
    }

    let mut p = buf.as_ptr();
    let end = unsafe { p.add(needed as usize) };

    let mut cores = 0;
    let mut threads = 0;

    while p < end {
        let info = unsafe { &*(p as *const SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX) };
        let size = info.Size as usize;

        match info.Relationship {
            r if r == RelationProcessorCore => {
                cores += 1;
                let mask = unsafe { info.Anonymous.Processor.GroupMask[0].Mask };
                threads += mask.count_ones() as usize;
            }
            _ => {}
        }
        p = unsafe { p.add(size) };
    }

    let (l1, l2, l3) = cache_size(cores, threads);

    (
        cores,
        threads,
        l1,
        l2,
        l3,
    )
}

fn read_registry_mhz() -> Option<u32> {
    unsafe {
        let mut h: HKEY = HKEY::default();
        let path = to_wstring("HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0");
        if RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(path.as_ptr()),
            0,
            KEY_READ,
            &mut h,
        )
        .is_err()
        {
            return None;
        }
        let name = to_wstring("~MHz");
        let mut buf = [0u8; 4];
        let mut len = buf.len() as u32;
        let mut ty = REG_VALUE_TYPE::default();
        if RegQueryValueExW(
            h,
            PCWSTR(name.as_ptr()),
            None,
            Some(&mut ty),
            Some(buf.as_mut_ptr()),
            Some(&mut len),
        )
        .is_err()
        {
            return None;
        }
        Some(u32::from_le_bytes(buf))
    }
}

fn read_microcode() -> Option<String> {
    use windows::Win32::System::Registry::*;
    unsafe {
        let mut h: HKEY = HKEY::default();
        let path = to_wstring("HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0");
        if RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(path.as_ptr()),
            0,
            KEY_READ,
            &mut h,
        )
        .is_err()
        {
            return None;
        }
        let name = to_wstring("Update Revision");
        let mut ty = REG_VALUE_TYPE::default();
        let mut buf = [0u8; 4];
        let mut len: u32 = buf.len() as u32;
        if RegQueryValueExW(
            h,
            PCWSTR(name.as_ptr()),
            None,
            Some(&mut ty),
            Some(buf.as_mut_ptr()),
            Some(&mut len),
        )
        .is_err()
        {
            return None;
        }
        let rev = u32::from_le_bytes(buf);
        Some(format!("0x{rev:08X}"))
    }
}

fn to_wstring(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub fn cache_size(cores: usize, threads: usize) -> (usize, usize, usize) {
    let max_ext = cpuid(0x8000_0000, 0).0;
    if max_ext >= 0x8000_001D {
        cache_size_universal(0x8000_001D, cores, threads)
    } else {
        cache_size_universal(4, cores, 0)
    }
}

fn cache_size_universal(func: u32, cores: usize, threads_shared: usize) -> (usize, usize, usize) {
    let mut cache = [0, 0, 0];
    for i in 0.. {
        let (eax, ebx, ecx, _) = cpuid(func, i);
        let cache_type = eax & 0x1F;
        if cache_type == 0 {
            break;
        }

        let level = (eax >> 5) & 0x7;
        let line_size = (ebx & 0xFFF) + 1;
        let partitions = ((ebx >> 12) & 0x3FF) + 1;
        let ways = ((ebx >> 22) & 0x3FF) + 1;
        let sets = ecx + 1;
        let size = ways * partitions * line_size * sets;

        let shared_logical = if threads_shared > 0 {
            threads_shared as u32 / (((eax >> 14) & 0xFFF) + 1)
        } else {
            1
        };

        match (cache_type, level) {
            (1, 1) => cache[0] += size,
            (2, 1) => cache[0] += size,
            (3, 2) => cache[1] += size,
            (3, 3) => cache[2] += size * shared_logical,
            _ => {}
        }
    }
    (
        cache[0] as usize * cores,
        cache[1] as usize * cores,
        cache[2] as usize,
    )
}