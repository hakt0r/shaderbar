/*
 ███████╗███████╗███╗   ██╗███████╗ ██████╗ ██████╗ ███████╗
 ██╔════╝██╔════╝████╗  ██║██╔════╝██╔═══██╗██╔══██╗██╔════╝
 ███████╗█████╗  ██╔██╗ ██║███████╗██║   ██║██████╔╝███████╗
 ╚════██║██╔══╝  ██║╚██╗██║╚════██║██║   ██║██╔══██╗╚════██║
 ███████║███████╗██║ ╚████║███████║╚██████╔╝██║  ██║███████║
 ╚══════╝╚══════╝╚═╝  ╚═══╝╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝
*/

use crate::state::NETWORK_INTERFACES;
use crate::state::TEMPERATURE;
use crate::state::*;
use chrono::Local;
use chrono::Timelike;
use std::cmp::max;

/*
 ███████╗████████╗██████╗ ██╗   ██╗ ██████╗████████╗
 ██╔════╝╚══██╔══╝██╔══██╗██║   ██║██╔════╝╚══██╔══╝
 ███████╗   ██║   ██████╔╝██║   ██║██║        ██║
 ╚════██║   ██║   ██╔══██╗██║   ██║██║        ██║
 ███████║   ██║   ██║  ██║╚██████╔╝╚██████╗   ██║
 ╚══════╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝  ╚═════╝   ╚═╝
*/

pub struct Sensors {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub cpu_count: u8,
    pub cpu_load: Vec<u8>,
    pub cpu_last_idle: Vec<u64>,
    pub cpu_last_total: Vec<u64>,
    pub cpu_temp: u8,
    pub gpu_temp: u8,
    pub cpu_fan: u8,
    pub gpu_fan: u8,
    pub bat: u8,
    pub bat_status: u8,
    pub net_count: u8,
    pub net_rx: Vec<u8>,
    pub net_last_rx: Vec<u64>,
    pub net_max_rx: Vec<u64>,
    pub net_tx: Vec<u8>,
    pub net_last_tx: Vec<u64>,
    pub net_max_tx: Vec<u64>,
    pub net_allowed: std::collections::HashMap<String, NetworkInterface>,
    pub mem: Vec<u8>,
    pub cpu_temp_path: String,
    pub gpu_temp_path: String,
    pub cpu_fan_path: String,
    pub gpu_fan_path: String,
}

/*
 ██╗███╗   ██╗██╗████████╗
 ██║████╗  ██║██║╚══██╔══╝
 ██║██╔██╗ ██║██║   ██║
 ██║██║╚██╗██║██║   ██║
 ██║██║ ╚████║██║   ██║
 ╚═╝╚═╝  ╚═══╝╚═╝   ╚═╝
*/

impl Sensors {
    pub fn new() -> Self {
        unsafe {
            Sensors {
                hour: 0,
                minute: 0,
                second: 0,
                cpu_count: 0,
                cpu_last_idle: vec![0u64; 0],
                cpu_last_total: vec![0u64; 0],
                cpu_load: vec![0u8; 0],
                cpu_temp: 0,
                gpu_temp: 0,
                cpu_fan: 0,
                gpu_fan: 0,
                bat: 0,
                bat_status: 0,
                net_count: 0,
                net_rx: vec![0u8; 0],
                net_last_rx: vec![0u64; 0],
                net_max_rx: vec![0u64; 0],
                net_tx: vec![0u8; 0],
                net_last_tx: vec![0u64; 0],
                net_max_tx: vec![0u64; 0],
                net_allowed: NETWORK_INTERFACES.as_ref().unwrap().clone(),
                mem: vec![0u8; 4],
                cpu_temp_path: TEMPERATURE.as_ref().unwrap().cpu.to_string(),
                gpu_temp_path: TEMPERATURE.as_ref().unwrap().gpu.to_string(),
                cpu_fan_path: FANS.as_ref().unwrap().cpu.to_string(),
                gpu_fan_path: FANS.as_ref().unwrap().gpu.to_string(),
            }
        }
    }

    /*
     ██████╗ ███████╗ █████╗ ██████╗
     ██╔══██╗██╔════╝██╔══██╗██╔══██╗
     ██████╔╝█████╗  ███████║██║  ██║
     ██╔══██╗██╔══╝  ██╔══██║██║  ██║
     ██║  ██║███████╗██║  ██║██████╔╝
     ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═════╝
    */

    pub fn read_lowfreq(&mut self) {
        let cpu_fan = read_number_from_file_sync(&self.cpu_fan_path).unwrap();
        let gpu_fan = read_number_from_file_sync(&self.gpu_fan_path).unwrap();
        let cpu_temp = read_number_from_file_sync(&self.cpu_temp_path).unwrap();
        let gpu_temp = read_number_from_file_sync(&self.gpu_temp_path).unwrap();
        let battery_percentage =
            read_number_from_file_sync("/sys/class/power_supply/BAT0/capacity").unwrap();
        let battery_status =
            read_string_from_file_sync("/sys/class/power_supply/BAT0/status").unwrap();
        self.cpu_temp = (255 * cpu_temp / 1000) as u8;
        self.gpu_temp = (255 * gpu_temp / 1000) as u8;
        self.cpu_fan = (255 * cpu_fan / 5000) as u8;
        self.gpu_fan = (255 * gpu_fan / 5000) as u8;
        self.bat = (255 * battery_percentage / 110) as u8;
        self.bat_status = if battery_status == "Charging" { 1 } else { 0 };
    }

    pub fn read(&mut self) {
        let cpu_load = read_string_from_file_sync("/proc/stat").unwrap();
        let net_load = read_string_from_file_sync("/proc/net/dev").unwrap();
        let mem_load = read_string_from_file_sync("/proc/meminfo").unwrap();
        let now = Local::now();
        self.hour = now.hour() as u8;
        self.minute = now.minute() as u8;
        self.second = now.second() as u8;
        self.read_cpu(cpu_load);
        self.read_network(net_load);
        self.read_memory(mem_load);
        crate::gl::uniform::update_uniforms();
    }

    /*
      ██████╗██████╗ ██╗   ██╗
     ██╔════╝██╔══██╗██║   ██║
     ██║     ██████╔╝██║   ██║
     ██║     ██╔═══╝ ██║   ██║
     ╚██████╗██║     ╚██████╔╝
      ╚═════╝╚═╝      ╚═════╝
    */

    fn read_cpu(&mut self, contents: String) {
        let lines = contents.lines();
        let len = lines.count();
        if self.cpu_load.len() < len {
            self.cpu_load.resize(len, 0);
            self.cpu_last_idle.resize(len, 0);
            self.cpu_last_total.resize(len, 0);
        }
        let mut is_first = true;
        let mut cpuid: usize = 0;
        for line in contents.lines() {
            if !line.starts_with("cpu") {
                continue;
            }
            if is_first {
                is_first = false;
                continue;
            }
            let cpu: Vec<u64> = line
                .split_whitespace()
                .skip(1)
                .map(|s| s.parse().unwrap())
                .collect();
            let user = cpu[0];
            let nice = cpu[1];
            let system = cpu[2];
            let idle = cpu[3];
            let iowait = cpu[4];
            let irq = cpu[5];
            let softirq = cpu[6];
            let steal = cpu[7];
            let guest = cpu[8];
            let guest_nice = cpu[9];
            let total =
                user + nice + system + idle + iowait + irq + softirq + steal + guest + guest_nice;
            let idle = idle + iowait;
            let last_total = self.cpu_last_total[cpuid];
            let last_idle = self.cpu_last_idle[cpuid];
            let relative_total = total - last_total;
            let relative_idle = idle - last_idle;
            let usage = if relative_total > 0 {
                255 * (relative_total - relative_idle) / relative_total
            } else {
                0
            };
            self.cpu_last_total[cpuid] = total;
            self.cpu_last_idle[cpuid] = idle;
            cpuid += 1;
            self.cpu_load[cpuid] = usage as u8;
        }
        self.cpu_count = cpuid as u8;
        if self.cpu_load.len() != cpuid {
            self.cpu_load.resize(cpuid, 0);
            self.cpu_last_idle.resize(cpuid, 0);
            self.cpu_last_total.resize(cpuid, 0);
        }
    }

    /*
     ███╗   ██╗███████╗████████╗
     ████╗  ██║██╔════╝╚══██╔══╝
     ██╔██╗ ██║█████╗     ██║
     ██║╚██╗██║██╔══╝     ██║
     ██║ ╚████║███████╗   ██║
     ╚═╝  ╚═══╝╚══════╝   ╚═╝
    */

    fn read_network(&mut self, contents: String) {
        let mut i = 0;
        let lines = contents.lines();
        let len = lines.count();
        if self.net_rx.len() < len {
            self.net_rx.resize(len, 0);
            self.net_tx.resize(len, 0);
            self.net_last_rx.resize(len, 0);
            self.net_last_tx.resize(len, 0);
            self.net_max_rx.resize(len, 0);
            self.net_max_tx.resize(len, 0);
        }
        for line in contents.lines() {
            let mut parts = line.split_whitespace();
            let interface = parts.next().unwrap().trim_end_matches(':');
            let config = self.net_allowed.get(interface);
            if !config.is_some() {
                continue;
            }
            let rx = parts.next().unwrap().parse::<u64>().unwrap();
            let tx = parts.next().unwrap().parse::<u64>().unwrap();
            let last_rx = self.net_last_rx[i];
            let last_tx = self.net_last_tx[i];
            let relative_rx = rx - last_rx;
            let relative_tx = tx - last_tx;
            let max_rx = max(self.net_max_rx[i], relative_rx);
            let max_tx = max(self.net_max_tx[i], relative_tx);
            self.net_max_rx[i] = max_rx;
            self.net_max_tx[i] = max_tx;
            self.net_rx[i] = (255 * relative_rx / max_rx) as u8;
            self.net_tx[i] = (255 * relative_tx / max_tx) as u8;
            i += 1;
        }
        self.net_count = i as u8;
        if self.net_rx.len() != i {
            self.net_rx.resize(i, 0);
            self.net_tx.resize(i, 0);
            self.net_last_rx.resize(i, 0);
            self.net_last_tx.resize(i, 0);
            self.net_max_rx.resize(i, 0);
            self.net_max_tx.resize(i, 0);
        }
    }

    /*
     ███╗   ███╗███████╗███╗   ███╗
     ████╗ ████║██╔════╝████╗ ████║
     ██╔████╔██║█████╗  ██╔████╔██║
     ██║╚██╔╝██║██╔══╝  ██║╚██╔╝██║
     ██║ ╚═╝ ██║███████╗██║ ╚═╝ ██║
     ╚═╝     ╚═╝╚══════╝╚═╝     ╚═╝
    */

    fn read_memory(&mut self, contents: String) {
        let mut total = 0;
        let mut free = 0;
        let mut buffers = 0;
        let mut cached = 0;
        for line in contents.lines() {
            let mut parts = line.split_whitespace();
            let key = parts.next().unwrap();
            let value = parts.next().unwrap().parse::<u64>().unwrap();
            match key {
                "MemTotal:" => total = value,
                "MemFree:" => free = value,
                "Buffers:" => buffers = value,
                "Cached:" => cached = value,
                _ => {}
            }
        }
        let used = total - free - buffers - cached;
        self.mem = vec![
            (255 * used / total) as u8,
            (255 * buffers / total) as u8,
            (255 * cached / total) as u8,
            (255 * free / total) as u8,
        ];
    }
}

/*
 ████████╗ ██████╗  ██████╗ ██╗     ███████╗
 ╚══██╔══╝██╔═══██╗██╔═══██╗██║     ██╔════╝
    ██║   ██║   ██║██║   ██║██║     ███████╗
    ██║   ██║   ██║██║   ██║██║     ╚════██║
    ██║   ╚██████╔╝╚██████╔╝███████╗███████║
    ╚═╝    ╚═════╝  ╚═════╝ ╚══════╝╚══════╝
*/

pub fn read_number_from_file_sync(path: &str) -> Result<u64, std::io::Error> {
    let contents = std::fs::read_to_string(path)?;
    Ok(contents.trim().parse().unwrap())
}

pub fn read_string_from_file_sync(path: &str) -> Result<String, std::io::Error> {
    let contents = std::fs::read_to_string(path)?;
    Ok(contents.trim().to_string())
}
