use crate::cpu_info::{CpuData, CpuInfo};
use std::fs;

pub struct LinuxCpuInfo(CpuData);
impl CpuInfo for LinuxCpuInfo {
    fn new() -> Self {
        let mut results = CpuData::default();
        results.arch = std::env::consts::ARCH.to_string();

        let (_, b, c, d) = cpuid(0, 0);
        let mut vendor = Vec::new();
        vendor.extend_from_slice(&b.to_le_bytes());
        vendor.extend_from_slice(&d.to_le_bytes());
        vendor.extend_from_slice(&c.to_le_bytes());
        results.vendor = String::from_utf8_lossy(&vendor).trim().to_string();

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

        results.stepping = (eax & 0xF) as u8;
        let model = ((eax >> 4) & 0xF) as u8;
        let family = ((eax >> 8) & 0xF) as u8;
        let ext_model = ((eax >> 16) & 0xF) as u8;
        let ext_family = ((eax >> 20) & 0xFF) as u8;
        results.family = if family == 0xF { family + ext_family } else { family };
        results.model = if family == 0x6 || family == 0xF { (ext_model << 4) + model } else { model };

        if (ecx & (1 << 23)) != 0 { results.flags.push("POP_CNT".into()); }
        if (edx & (1 << 25)) != 0 { results.flags.push("SSE".into()); }
        if (edx & (1 << 26)) != 0 { results.flags.push("SSE2".into()); }
        if (ecx & (1 << 0)) != 0 { results.flags.push("SSE3".into()); }
        if (ecx & (1 << 9)) != 0 { results.flags.push("SSSE3".into()); }
        if (ecx & (1 << 19)) != 0 { results.flags.push("SSE4.1".into()); }
        if (ecx & (1 << 20)) != 0 { results.flags.push("SSE4.2".into()); }
        if (ecx & (1 << 25)) != 0 { results.flags.push("AES".into()); }
        if (ecx & (1 << 28)) != 0 { results.flags.push("AVX".into()); }

        let (_, ebx, _, _) = cpuid(7, 0);
        if (ebx & (1 << 5)) != 0 { results.flags.push("AVX2".into()); }
        if (ebx & (1 << 8)) != 0 { results.flags.push("BMI2".into()); }
        if (ebx & (1 << 16)) != 0 { results.flags.push("AVX512F".into()); }
        if (ebx & (1 << 30)) != 0 { results.flags.push("AVX512BW".into()); }
        if (ebx & (1 << 31)) != 0 { results.flags.push("AVX512V1".into()); }

        (
            results.cores,
            results.threads,
            results.l1_cache,
            results.l2_cache,
            results.l3_cache,
        ) = topo_and_caches();

        (results.clock_speed, results.clock_speed_turbo, _, _) = cpuid(0x16, 0);
        if results.clock_speed == 0 {
            if let Some(base_clock) = read_base_mhz() {
                results.clock_speed = base_clock
            }
        }

        results.microcode = read_microcode().unwrap_or_default();

        Self(results)
    }

    fn static_data(&self) -> &CpuData {
        &self.0
    }

    fn uptime(&self) -> f64 {
        if let Ok(s) = fs::read_to_string("/proc/uptime") {
            if let Some(first) = s.split_whitespace().next() {
                if let Ok(v) = first.parse::<f64>() {
                    return v;
                }
            }
        }
        0.0
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
    let mut logical = 0u32;
    let mut smt = 0u32;

    let mut try_leaf = |leaf: u32| -> bool {
        let max_basic = cpuid(0, 0).0;
        if max_basic < leaf { return false; }
        let mut sub = 0u32;
        loop {
            let (ea, eb, ec, _) = cpuid(leaf, sub);
            let level_type = (ec >> 8) & 0xff;
            let level_count = eb & 0xffff;
            if level_count == 0 { break; }
            if level_type == 1 { smt = level_count; }
            if level_type == 2 { logical = level_count; }
            if (ea & 0x1f) == 0 { break; }
            sub += 1;
        }
        logical != 0
    };

    let ok = try_leaf(0x1f) || try_leaf(0x0b);

    let (cores, threads) = if ok {
        let smt_eff = smt.max(1);
        let phys = (logical / smt_eff).max(1);
        (phys as usize, logical as usize)
    } else {
        let vendor = {
            let (_, b, c, d) = cpuid(0, 0);
            let mut v = Vec::new();
            v.extend_from_slice(&b.to_le_bytes());
            v.extend_from_slice(&d.to_le_bytes());
            v.extend_from_slice(&c.to_le_bytes());
            String::from_utf8_lossy(&v).to_string()
        };
        if cpuid(0x8000_0000, 0).0 >= 0x8000_0008 && vendor.contains("AuthenticAMD") {
            let (_, eb, _, _) = cpuid(1, 0);
            let lpp = ((eb >> 16) & 0xff) as u32;
            let phys = ((cpuid(0x8000_0008, 0).2 & 0xff) + 1) as u32;
            let l = if lpp != 0 { lpp } else { phys };
            (phys as usize, l as usize)
        } else if cpuid(0, 0).0 >= 4 && vendor.contains("GenuineIntel") {
            let mut i = 0u32;
            let mut max_phys_minus1 = 0u32;
            loop {
                let (eax, _, _, _) = cpuid(4, i);
                if (eax & 0x1f) == 0 { break; }
                let c = (eax >> 26) & 0x3f;
                if c > max_phys_minus1 { max_phys_minus1 = c; }
                i += 1;
            }
            let phys = (max_phys_minus1 + 1).max(1);
            let lpp = ((cpuid(1, 0).1 >> 16) & 0xff) as u32;
            let l = if lpp != 0 { lpp } else { phys };
            (phys as usize, l as usize)
        } else {
            let lpp = ((cpuid(1, 0).1 >> 16) & 0xff) as usize;
            let l = if lpp != 0 { lpp } else { 1 };
            (1, l)
        }
    };

    let (l1, l2, l3) = cache_size(cores, threads);
    (cores, threads, l1, l2, l3)
}

fn read_base_mhz() -> Option<u32> {
    if let Ok(s) = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/base_frequency") {
        if let Ok(khz) = s.trim().parse::<u32>() {
            return Some(khz / 1000);
        }
    }
    if let Ok(s) = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq") {
        if let Ok(khz) = s.trim().parse::<u32>() {
            return Some(khz / 1000);
        }
    }
    if let Ok(s) = fs::read_to_string("/proc/cpuinfo") {
        for line in s.lines() {
            if let Some(rest) = line.strip_prefix("cpu MHz") {
                if let Some(val) = rest.split(':').nth(1) {
                    if let Ok(mhzf) = val.trim().parse::<f64>() {
                        return Some(mhzf.round() as u32);
                    }
                }
            }
        }
    }
    None
}

fn read_microcode() -> Option<String> {
    if let Ok(s) = fs::read_to_string("/proc/cpuinfo") {
        for line in s.lines() {
            if let Some(rest) = line.strip_prefix("microcode") {
                if let Some(val) = rest.split(':').nth(1) {
                    let t = val.trim();
                    if t.starts_with("0x") {
                        return Some(format!("{:#010X}", u32::from_str_radix(&t[2..], 16).ok()?));
                    } else if let Ok(v) = t.parse::<u32>() {
                        return Some(format!("{:#010X}", v));
                    }
                }
            }
        }
    }
    None
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
    (cache[0] as usize * cores, cache[1] as usize * cores, cache[2] as usize)
}
