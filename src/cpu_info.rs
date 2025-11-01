#[cfg(target_os = "windows")]
mod windows_cpu_info;
#[cfg(target_os = "linux")]
mod linux_cpu_info;

#[allow(unused)]
#[cfg(target_os = "windows")]
pub use windows_cpu_info::WindowsCpuInfo;
#[allow(unused)]
#[cfg(target_os = "linux")]
pub use linux_cpu_info::LinuxCpuInfo;

pub trait CpuInfo {
    fn new() -> Self;

    fn static_data(&self) -> &CpuData;

    fn uptime(&self) -> f64;
}

#[derive(Debug, Clone, Default)]
pub struct CpuData {
    brand: String,
    vendor: String,
    arch: String,
    family: u8,
    model: u8,
    stepping: u8,
    microcode: String,
    l1_cache: usize,
    l2_cache: usize,
    l3_cache: usize,
    cores: usize,
    threads: usize,
    flags: Vec<String>,
    clock_speed: u32,
    clock_speed_turbo: u32,
}

impl CpuData {
    pub fn brand(&self) -> &String {
        &self.brand
    }

    pub fn vendor(&self) -> &String {
        &self.vendor
    }

    pub fn architecture(&self) -> &String {
        &self.arch
    }

    pub fn family(&self) -> u8 {
        self.family
    }

    pub fn model(&self) -> u8 {
        self.model
    }

    pub fn stepping(&self) -> u8 {
        self.stepping
    }

    pub fn microcode_version(&self) -> &String {
        &self.microcode
    }

    pub fn l1_cache(&self) -> usize {
        self.l1_cache
    }

    pub fn l2_cache(&self) -> usize {
        self.l2_cache
    }

    pub fn l3_cache(&self) -> usize {
        self.l3_cache
    }

    pub fn cores(&self) -> usize {
        self.cores
    }

    pub fn threads(&self) -> usize {
        self.threads
    }

    pub fn flags(&self) -> &Vec<String> {
        &self.flags
    }

    pub fn clock_speed(&self) -> u32 {
        self.clock_speed
    }

    pub fn clock_speed_turbo(&self) -> u32 {
        self.clock_speed_turbo
    }
}
