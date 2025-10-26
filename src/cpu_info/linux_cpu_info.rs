use crate::cpu_info::{CpuData, CpuInfo};

pub struct LinuxCpuInfo(CpuData);
// impl CpuInfo for LinuxCpuInfo {
//     fn init(&mut self) {

//     }

//     fn static_data(&self) -> &CpuData {
//          &self.0
//     }

//     fn current_clock_speed(&self) -> f32 {

//     }

//     fn min_clock_speed(&self) -> f32 {

//     }

//     fn max_clock_speed(&self) -> f32 {

//     }

//     fn usage(&self) -> f32 {

//     }

//     fn scaling_governor(&self) -> String {

//     }

//     fn uptime(&self) -> String {

//     }
// }
