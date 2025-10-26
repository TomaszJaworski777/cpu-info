use crate::cpu_info::CpuInfo;

mod cpu_info;

fn main() {
    let cpu_info = if cfg!(target_os = "windows") {
        use crate::cpu_info::WindowsCpuInfo;
        WindowsCpuInfo::new()
    } else if cfg!(target_os = "linux") {
        use crate::cpu_info::WindowsCpuInfo;
        WindowsCpuInfo::new() //TODO: Change to linux
    } else {
        panic!("Target OS not supported!")
    };

    print_data(cpu_info);
}

fn print_data<Cpu>(info: Cpu)
where
    Cpu: CpuInfo,
{
    println!("Arch:     {}", info.static_data().architecture());
    println!("Vendor:   {}", info.static_data().vendor());
    println!("Brand:    {}", info.static_data().brand());
    println!("Family:   {}", info.static_data().family());
    println!("Model:    {}", info.static_data().model());
    println!("Stepping: {}", info.static_data().stepping());
    println!("Flags:    {}\n", info.static_data().flags().join(", "));

    println!("Cores:    {}", info.static_data().cores());
    println!("Threads:  {}", info.static_data().threads());
    println!("L1 Cache: {}", info.static_data().l1_cache());
    println!("L2 Cache: {}", info.static_data().l2_cache());
    println!("L3 Cache: {}\n", info.static_data().l3_cache());

    println!("Clock speed:       {}", info.static_data().clock_speed());
    println!(
        "Clock speed turno: {}\n",
        info.static_data().clock_speed_turbo()
    );

    println!(
        "Microcode Version: {}\n",
        info.static_data().microcode_version()
    );

    println!("Uptime:     {}", info.uptime());
    println!("Power Plan: {}", info.power_plan());
}
