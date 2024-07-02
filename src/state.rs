use std::collections::HashMap;

use serde::Deserialize;

pub static mut NETWORK_INTERFACES: Option<HashMap<String, NetworkInterface>> = None;
pub static mut TEMPERATURE: Option<Temperature> = None;
pub static mut FANS: Option<Fans> = None;

/*
███████╗████████╗ █████╗ ████████╗███████╗
██╔════╝╚══██╔══╝██╔══██╗╚══██╔══╝██╔════╝
███████╗   ██║   ███████║   ██║   █████╗
╚════██║   ██║   ██╔══██║   ██║   ██╔══╝
███████║   ██║   ██║  ██║   ██║   ███████╗
╚══════╝   ╚═╝   ╚═╝  ╚═╝   ╚═╝   ╚══════╝
*/

pub struct State {
    pub sensor_index: u64,
    pub prev_idle: Vec<u64>,
    pub prev_total: Vec<u64>,
    pub prev_rx_bytes: Vec<u64>,
    pub prev_tx_bytes: Vec<u64>,
    pub max_tx: Vec<u64>,
    pub max_rx: Vec<u64>,
    pub history: Vec<Vec<f64>>,
}

pub fn state() -> &'static mut State {
    pub static mut STATE: Option<State> = None;
    let mut map = HashMap::new();
    map.insert(
        "enx207bd2ddfd75".to_string(),
        NetworkInterface {
            type_: "ethernet".to_string(),
            icon: "".to_string(),
        },
    );
    map.insert(
        "wlp4s0".to_string(),
        NetworkInterface {
            type_: "wifi".to_string(),
            icon: "".to_string(),
        },
    );
    unsafe {
        NETWORK_INTERFACES = Some(map);
        TEMPERATURE = Some(Temperature {
            cpu: "/sys/devices/platform/thinkpad_hwmon/hwmon/hwmon7/temp1_input".to_string(),
            gpu: "/sys/devices/platform/thinkpad_hwmon/hwmon/hwmon7/temp2_input".to_string(),
        });
        FANS = Some(Fans {
            cpu: "/sys/devices/platform/thinkpad_hwmon/hwmon/hwmon7/fan1_input".to_string(),
            gpu: "/sys/devices/platform/thinkpad_hwmon/hwmon/hwmon7/fan2_input".to_string(),
        });
        STATE = Some(State {
            sensor_index: 0,
            prev_idle: Vec::new(),
            prev_total: Vec::new(),
            prev_rx_bytes: Vec::new(),
            prev_tx_bytes: Vec::new(),
            max_tx: Vec::new(),
            max_rx: Vec::new(),
            history: Vec::new(),
        });
        let state = STATE.as_mut().unwrap();
        state.prev_idle.resize(64, 0);
        state.prev_total.resize(64, 0);
        state.prev_rx_bytes.resize(64, 0);
        state.prev_tx_bytes.resize(64, 0);
        state.max_tx.resize(64, 0);
        state.max_rx.resize(64, 0);
        let mut history: Vec<Vec<f64>> = Vec::new();
        for _ in 0..64 {
            history.push(Vec::new());
        }
        state.history = history;
        return STATE.as_mut().unwrap();
    }
}

/*
███████╗████████╗██████╗ ██╗   ██╗ ██████╗████████╗███████╗
██╔════╝╚══██╔══╝██╔══██╗██║   ██║██╔════╝╚══██╔══╝██╔════╝
███████╗   ██║   ██████╔╝██║   ██║██║        ██║   ███████╗
╚════██║   ██║   ██╔══██╗██║   ██║██║        ██║   ╚════██║
███████║   ██║   ██║  ██║╚██████╔╝╚██████╗   ██║   ███████║
╚══════╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝  ╚═════╝   ╚═╝   ╚══════╝
*/

#[derive(Debug, Clone, Deserialize)]
pub struct NetworkInterface {
    pub type_: String,
    pub icon: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Temperature {
    pub cpu: String,
    pub gpu: String,
}

#[derive(Debug, Clone, Deserialize)]

pub struct Fans {
    pub cpu: String,
    pub gpu: String,
}
