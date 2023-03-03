use sysinfo::{System, SystemExt, CpuExt};

fn main() {
    let mut system_info = System::new_all();

    system_info.refresh_all();

    println!("System name:           {:?}", match system_info.name() {
        Some(v) => v,
        None => "Something went wrong...".to_owned()
    });
    println!("System kernel version: {:?}", match system_info.kernel_version() {
        Some(v) => v,
        None => "Something went wrong...".to_owned()
    });
    println!("System OS version:     {:?}", match system_info.os_version() {
        Some(v) => v,
        None => "Something went wrong...".to_owned()
    });
    println!("System host name:      {:?}", match system_info.host_name() {
        Some(v) => v,
        None => "Something went wrong...".to_owned()
    });
    println!("NB CPU THREADs:        {}", system_info.cpus().len());

    for _ in 1..=10 {
        print!("{}\r", ansi_escapes::EraseLine);
        system_info.refresh_cpu();
        for cpu in system_info.cpus() {
            print!("{}% ", cpu.cpu_usage());
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
