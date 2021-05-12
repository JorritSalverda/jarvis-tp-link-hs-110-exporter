use crate::model::{Config, Measurement, MetricType, Sample, SampleType};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::error::Error;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub struct HS110ClientConfig {
    timeout_seconds: u64,
}

impl HS110ClientConfig {
    pub fn new(timeout_seconds: u64) -> Result<Self, Box<dyn Error>> {
        println!(
            "HS110ClientConfig::new(timeout_seconds: {})",
            timeout_seconds
        );
        Ok(Self { timeout_seconds })
    }

    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let timeout_seconds: u64 = env::var("TIMEOUT_SECONDS")
            .unwrap_or("10".to_string())
            .parse()?;

        Self::new(timeout_seconds)
    }
}

pub struct HS110Client {
    config: HS110ClientConfig,
}

impl HS110Client {
    pub fn new(config: HS110ClientConfig) -> Self {
        Self { config }
    }

    pub fn get_measurement(
        &self,
        config: Config,
        last_measurement: Option<Measurement>,
    ) -> Result<Measurement, Box<dyn Error>> {
        println!("Reading measurement from hs-110 devices...");

        let mut measurement = Measurement {
            id: Uuid::new_v4().to_string(),
            source: String::from("jarvis-tp-link-hs-110-exporter"),
            location: config.location.clone(),
            samples: Vec::new(),
            measured_at_time: Utc::now(),
        };

        println!("Discovering devices...");
        let devices = self.discover_devices()?;

        for device in devices.iter() {
            match &device.system {
                Some(system) => {
                    match &device.e_meter {
                        Some(e_meter) => {
                            // counter
                            measurement.samples.push(Sample {
                                entity_type: config.entity_type,
                                entity_name: config.entity_name.clone(),
                                sample_type: SampleType::ElectricityConsumption,
                                sample_name: system.info.alias.clone(),
                                metric_type: MetricType::Counter,
                                value: e_meter.real_time.total_watt_hour * 3600.0,
                            });

                            // gauge
                            measurement.samples.push(Sample {
                                entity_type: config.entity_type,
                                entity_name: config.entity_name.clone(),
                                sample_type: SampleType::ElectricityConsumption,
                                sample_name: system.info.alias.clone(),
                                metric_type: MetricType::Gauge,
                                value: e_meter.real_time.power_milli_watt / 1000.0,
                            });
                        }
                        None => (),
                    }
                }
                None => (),
            }
        }

        match last_measurement {
            Some(lm) => {
                measurement.samples = self.sanitize_samples(measurement.samples, lm.samples)
            }
            None => {}
        }

        println!("Read measurement from hs-110 devices");

        Ok(measurement)
    }

    fn discover_devices(&self) -> Result<Vec<DeviceInfoResponse>, Box<dyn Error>> {
        // init udp socket
        let broadcast_address: SocketAddr = "255.255.255.255:9999".parse()?;
        let from_address: SocketAddr = "0.0.0.0:8755".parse()?;
        let socket = UdpSocket::bind(from_address)?;
        socket.set_read_timeout(Some(Duration::new(self.config.timeout_seconds.clone(), 0)))?;
        socket.set_broadcast(true)?;

        // broadcast request for device info
        let request: DeviceInfoRequest = Default::default();
        let request = serde_json::to_vec(&request)?;
        let request = self.encrypt(request);
        socket.send_to(&request, broadcast_address)?;

        // await all responses
        let mut read_buffer: Vec<u8> = vec![0; 2048];
        let mut devices = Vec::new();
        let start = Instant::now();
        let timeout = Duration::new(self.config.timeout_seconds.clone(), 0);

        while let Ok((number_of_bytes, src_addr)) = socket.recv_from(&mut read_buffer) {
            println!(
                "Received {} bytes from address {}",
                number_of_bytes, src_addr
            );

            let response: Vec<u8> = read_buffer[..number_of_bytes].to_vec();
            let response = self.decrypt(response);
            let response: DeviceInfoResponse = serde_json::from_slice(&response)?;

            devices.push(response);

            if start.elapsed() > timeout {
                break;
            }
        }

        Ok(devices)
    }

    fn encrypt(&self, input: Vec<u8>) -> Vec<u8> {
        let key = b"\xAB";

        // s := string(input)

        // key := byte(0xAB)
        // b := make(Vec<u8>, len(s))
        // for i := 0; i < len(s); i++ {
        //     b[i] = s[i] ^ key
        //     key = b[i]
        // }
        // return b
        vec![]
    }

    fn decrypt(&self, b: Vec<u8>) -> Vec<u8> {
        // k := byte(0xAB)
        // var newKey byte
        // for i := 0; i < len(b); i++ {
        //     newKey = b[i]
        //     b[i] = b[i] ^ k
        //     k = newKey
        // }

        // return b
        vec![]
    }

    fn sanitize_samples(
        &self,
        current_samples: Vec<Sample>,
        last_samples: Vec<Sample>,
    ) -> Vec<Sample> {
        let mut sanitized_samples: Vec<Sample> = Vec::new();

        for current_sample in current_samples.into_iter() {
            // check if there's a corresponding sample in lastSamples and see if the difference with it's value isn't too large
            let mut sanitize = false;
            for last_sample in last_samples.iter() {
                if current_sample.entity_type == last_sample.entity_type
                    && current_sample.entity_name == last_sample.entity_name
                    && current_sample.sample_type == last_sample.sample_type
                    && current_sample.sample_name == last_sample.sample_name
                    && current_sample.metric_type == last_sample.metric_type
                {
                    if current_sample.metric_type == MetricType::Counter
                        && current_sample.value / last_sample.value > 1.1
                    {
                        sanitize = true;
                        println!("Value for {} is more than 10 percent larger than the last sampled value {}, keeping previous value instead", current_sample.sample_name, last_sample.value);
                        sanitized_samples.push(last_sample.clone());
                    }

                    break;
                }
            }

            if !sanitize {
                sanitized_samples.push(current_sample);
            }
        }

        sanitized_samples
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct DeviceInfoRequest {
    #[serde(rename = "system")]
    system: System,
    #[serde(rename = "emeter")]
    e_meter: EMeter,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeviceInfoResponse {
    #[serde(rename = "system")]
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<System>,
    #[serde(rename = "emeter")]
    #[serde(skip_serializing_if = "Option::is_none")]
    e_meter: Option<EMeter>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct System {
    #[serde(rename = "get_sysinfo")]
    info: SystemInfo,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct EMeter {
    #[serde(rename = "get_realtime")]
    real_time: RealTimeEnergy,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct SystemInfo {
    #[serde(rename = "active_mode")]
    mode: String,
    #[serde(rename = "alias")]
    alias: String,
    #[serde(rename = "dev_name")]
    product: String,
    #[serde(rename = "device_id")]
    device_id: String,
    #[serde(rename = "err_code")]
    error_code: i32,
    #[serde(rename = "feature")]
    features: String,
    #[serde(rename = "fwId")]
    firmware_id: String,
    #[serde(rename = "hwId")]
    hardware_id: String,
    #[serde(rename = "hw_ver")]
    hardware_version: String,
    #[serde(rename = "icon_hash")]
    icon_hash: String,
    #[serde(rename = "latitude")]
    gps_latitude: f32,
    #[serde(rename = "longitude")]
    gps_longitude: f32,
    #[serde(rename = "led_off")]
    led_off: u8,
    #[serde(rename = "mac")]
    mac: String,
    #[serde(rename = "model")]
    model: String,
    #[serde(rename = "oemId")]
    oem_id: String,
    #[serde(rename = "on_time")]
    on_time: u32,
    #[serde(rename = "relay_state")]
    relay_on: u8,
    #[serde(rename = "rssi")]
    rssi: i32,
    #[serde(rename = "sw_ver")]
    software_version: String,
    #[serde(rename = "type")]
    product_type: String,
    #[serde(rename = "updating")]
    updating: u8,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct RealTimeEnergy {
    #[serde(rename = "err_code")]
    error_code: u8,
    #[serde(rename = "power_mw")]
    power_milli_watt: f64,
    #[serde(rename = "voltage_mv")]
    voltage_milli_volt: f64,
    #[serde(rename = "current_ma")]
    current_milli_ampere: f64,
    #[serde(rename = "total_wh")]
    total_watt_hour: f64,
}
