use sysinfo::System;

pub struct PerformanceManager {
    cpus_count: f32,
    system: System,
}

impl PerformanceManager {
    pub fn new() -> Self {
        let system = System::new_all();
        let cpus_count = system.cpus().len();

        tracing::info!("performance manager found {} CPU cores", cpus_count);

        Self {
            cpus_count: cpus_count as f32,
            system,
        }
    }

    pub fn get_total_cpu_usage(&self) -> f32 {
        self.system.global_cpu_info().cpu_usage() / self.cpus_count
    }
}
