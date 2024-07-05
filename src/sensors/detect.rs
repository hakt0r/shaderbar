use crate::utils::early_continue;
use std::collections::HashMap;

/*
 ███████╗██╗   ██╗███████╗███████╗███████╗
 ██╔════╝╚██╗ ██╔╝██╔════╝██╔════╝██╔════╝
 ███████╗ ╚████╔╝ ███████╗█████╗  ███████╗
 ╚════██║  ╚██╔╝  ╚════██║██╔══╝  ╚════██║
 ███████║   ██║   ███████║██║     ███████║
 ╚══════╝   ╚═╝   ╚══════╝╚═╝     ╚══════╝
*/

pub async fn detect_sensors() -> HashMap<String, Value> {
    let mut detected: HashMap<String, Value> = HashMap::new();
    detected.insert("cpu.count".to_string(), Value::U64(0));
    detected.insert("gpu.count".to_string(), Value::U64(0));
    detected.insert(
        "cpu.count".to_string(),
        Value::U64(command_line("nproc").await.parse().unwrap()),
    );
    let labels = labels().await;
    for (k, _) in match_regex(r"coretemp|k10temp|cpu|CPU", labels.clone()) {
        let temp_path = format!("{}_input", k.replace("_label", ""));
        let fan_path = temp_path.replace("temp", "fan");
        add(&mut detected, "cpu.temp", &temp_path);
        add(&mut detected, "cpu.fan", &fan_path);
    }
    for (k, _) in match_regex(r"amdgpu|nouveau|nvidia|intel|GPU", labels) {
        let temp_path = format!("{}_input", k.replace("_label", ""));
        let fan_path = temp_path.replace("temp", "fan");
        add(&mut detected, "gpu.temp", &temp_path);
        add(&mut detected, "gpu.fan", &fan_path);
    }
    detected = detect_amdgpu(detected).await;
    detected = detect_network(detected).await;
    detected = detect_battery(detected).await;
    detected
}

pub async fn labels() -> HashMap<String, Value> {
    let mut map: HashMap<String, Value> = HashMap::new();
    let keys = glob::glob("/sys/class/hwmon/hwmon*/*_label")
        .unwrap()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    for key in keys {
        let key = key.to_str().unwrap().to_string();
        let value = tokio::fs::read_to_string(&key)
            .await
            .unwrap()
            .trim()
            .to_string();
        map.insert(key, Value::String(value));
    }
    map
}

/*
  █████╗ ███╗   ███╗██████╗
 ██╔══██╗████╗ ████║██╔══██╗
 ███████║██╔████╔██║██║  ██║
 ██╔══██║██║╚██╔╝██║██║  ██║
 ██║  ██║██║ ╚═╝ ██║██████╔╝
 ╚═╝  ╚═╝╚═╝     ╚═╝╚═════╝
*/

async fn detect_amdgpu(mut detected: HashMap<String, Value>) -> HashMap<String, Value> {
    let gpu_count_key = &"gpu.count".to_string();
    let mut gpus = detected.get(gpu_count_key).unwrap().to_u64();
    for line in command_lines("find /sys/devices/pci* -name gpu_busy_percent").await {
        let path = line.trim().to_string();
        let key = format!("gpu[{}].usage", gpus);
        let dirname = dirname(path.clone());
        detected.insert(key, Value::String(path.clone()));
        let udevadm_call = format!(
            "udevadm info -q property -p {} --property ID_MODEL_FROM_DATABASE",
            dirname
        );
        let model = command_line(&udevadm_call).await.split_off(23);
        let key = format!("gpu[{}].model", gpus);
        detected.insert(key, Value::String(model));
        gpus += 1;
    }
    detected.insert("gpu.count".to_string(), Value::U64(gpus));
    detected
}

/*
 ███╗   ██╗███████╗████████╗██╗    ██╗ ██████╗ ██████╗ ██╗  ██╗
 ████╗  ██║██╔════╝╚══██╔══╝██║    ██║██╔═══██╗██╔══██╗██║ ██╔╝
 ██╔██╗ ██║█████╗     ██║   ██║ █╗ ██║██║   ██║██████╔╝█████╔╝
 ██║╚██╗██║██╔══╝     ██║   ██║███╗██║██║   ██║██╔══██╗██╔═██╗
 ██║ ╚████║███████╗   ██║   ╚███╔███╔╝╚██████╔╝██║  ██║██║  ██╗
 ╚═╝  ╚═══╝╚══════╝   ╚═╝    ╚══╝╚══╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝
*/

async fn detect_network(mut detected: HashMap<String, Value>) -> HashMap<String, Value> {
    for line in command_lines("nmcli device").await {
        let words = line.split_whitespace().collect::<Vec<&str>>();
        early_continue!(words.len() < 3 || words[2] != "connected");
        match words[1] {
            "wifi" => {
                add(&mut detected, "wifi.interface", words[0]);
                add(&mut detected, "wifi.connection", words[3]);
            }
            "ethernet" => {
                add(&mut detected, "ethernet.interface", words[0]);
                add(&mut detected, "ethernet.connection", words[3]);
            }
            "gsm" => {
                add(&mut detected, "gsm.interface", words[0]);
                add(&mut detected, "gsm.connection", words[3]);
            }
            _ => (),
        }
    }
    detected
}

fn add(detected: &mut HashMap<String, Value>, key: &str, value: &str) {
    detected.insert(key.to_string(), Value::String(value.to_string()));
}

/*
 ██████╗  █████╗ ████████╗████████╗███████╗██████╗ ██╗   ██╗
 ██╔══██╗██╔══██╗╚══██╔══╝╚══██╔══╝██╔════╝██╔══██╗╚██╗ ██╔╝
 ██████╔╝███████║   ██║      ██║   █████╗  ██████╔╝ ╚████╔╝
 ██╔══██╗██╔══██║   ██║      ██║   ██╔══╝  ██╔══██╗  ╚██╔╝
 ██████╔╝██║  ██║   ██║      ██║   ███████╗██║  ██║   ██║
 ╚═════╝ ╚═╝  ╚═╝   ╚═╝      ╚═╝   ╚══════╝╚═╝  ╚═╝   ╚═╝
*/

async fn detect_battery(mut map: HashMap<String, Value>) -> HashMap<String, Value> {
    for line in command_lines("ls /sys/class/power_supply/*/capacity").await {
        let path = line.trim().to_string();
        add(&mut map, "battery.capacity", &path);
        add(
            &mut map,
            "battery.status",
            &path.replace("/capacity", "/status"),
        );
        break;
    }
    map
}

/*
 ██╗  ██╗███████╗██╗     ██████╗ ███████╗██████╗ ███████╗
 ██║  ██║██╔════╝██║     ██╔══██╗██╔════╝██╔══██╗██╔════╝
 ███████║█████╗  ██║     ██████╔╝█████╗  ██████╔╝███████╗
 ██╔══██║██╔══╝  ██║     ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║
 ██║  ██║███████╗███████╗██║     ███████╗██║  ██║███████║
 ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝
*/

#[allow(dead_code)]
async fn fs_exists(path: &str) -> bool {
    tokio::fs::metadata(path).await.is_err()
}

fn match_regex(regex: &str, labels: HashMap<String, Value>) -> HashMap<String, Value> {
    let regex = regex::Regex::new(regex).unwrap();
    labels
        .into_iter()
        .filter(|(_, v)| regex.is_match(v.to_string().as_str()))
        .collect::<HashMap<String, Value>>()
}

async fn command_line(command: &str) -> String {
    String::from_utf8(
        tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .await
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string()
}

async fn command_lines(command: &str) -> Vec<String> {
    String::from_utf8(
        tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .await
            .unwrap()
            .stdout,
    )
    .unwrap()
    .lines()
    .map(|x| x.to_string())
    .collect::<Vec<String>>()
}

fn dirname(path: String) -> String {
    std::path::Path::new(&path)
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

#[allow(dead_code)]
pub async fn read_string_from_file(path: &str) -> String {
    tokio::fs::read_to_string(path)
        .await
        .expect(format!("Failed to read file: {}", path).as_str())
        .trim()
        .to_string()
}

/*
 ██╗   ██╗ █████╗ ██╗     ██╗   ██╗███████╗
 ██║   ██║██╔══██╗██║     ██║   ██║██╔════╝
 ██║   ██║███████║██║     ██║   ██║█████╗
 ╚██╗ ██╔╝██╔══██║██║     ██║   ██║██╔══╝
  ╚████╔╝ ██║  ██║███████╗╚██████╔╝███████╗
   ╚═══╝  ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚══════╝
*/
pub enum Value {
    U64(u64),
    String(String),
}

impl Value {
    pub fn to_u64(&self) -> u64 {
        match self {
            Value::U64(x) => *x,
            Value::String(x) => x.parse::<u64>().ok().unwrap(),
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Value::U64(x) => x.to_string(),
            Value::String(x) => x.to_string(),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::U64(x) => write!(f, "{}", x),
            Value::String(x) => write!(f, "{}", x),
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::U64(x) => write!(f, "{}", x),
            Value::String(x) => write!(f, "{}", x),
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::U64(x) => Value::U64(*x),
            Value::String(x) => Value::String(x.clone()),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }
}
