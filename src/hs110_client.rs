use crate::model::{Config, Measurement, MetricType, Sample, SampleType};

use std::net::{SocketAddr, UdpSocket};
use chrono::Utc;
use std::env;
use std::error::Error;
use std::time::Duration;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

pub struct HS110ClientConfig {
    timeout_seconds: u64,
}

impl HS110ClientConfig {
    pub fn new(timeout_seconds: u64) -> Result<Self, Box<dyn Error>> {
        println!("HS110ClientConfig::new(timeout_seconds: {})", timeout_seconds);
        Ok(Self { timeout_seconds })
    }

    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let timeout_seconds: u64 = env::var("TIMEOUT_SECONDS").unwrap_or("10".to_string()).parse()?;

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

            match device.info {
                Some(info) => {
                    match info.system {
                        Some(system) => {
                            match info.e_meter {
                                Some(e_meter) => {
                                    // counter
                                    measurement.samples.push(Sample {
                                        entity_type: config.entity_type,
                                        entity_name: config.entity_name.clone(),
                                        sample_type: SampleType::ElectricityConsumption,
                                        sample_name: system.info.alias,
                                        metric_type: MetricType::Counter,
                                        value: e_meter.real_time.total_watt_hour * 3600.0,
                                    });

                                    // gauge
                                    measurement.samples.push(Sample {
                                        entity_type: config.entity_type,
                                        entity_name: config.entity_name.clone(),
                                        sample_type: SampleType::ElectricityConsumption,
                                        sample_name: system.info.alias,
                                        metric_type: MetricType::Gauge,
                                        value: e_meter.real_time.power_milli_watt / 1000.0,
                                    });
                                },
                                None => ()
                            }
                        },
                        None => ()
                    }
                },
                None => ()
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

    fn discover_devices(&self)  -> Result<Vec<Device>, Box<dyn Error>> {
        
        let devices = Vec::new();

        let request = DeviceInfoRequest{
            
        };

        let broadcast_address: SocketAddr = "255.255.255.255:9999".parse()?;
        let from_address: SocketAddr = "0.0.0.0:8755".parse()?;

        let socket = UdpSocket::bind(from_address)?;
        socket.set_read_timeout(Some(Duration::new(self.config.timeout_seconds.clone(), 0)))?;
        socket.set_broadcast(true)?; 

        let mut write_buf: Vec<u8> = vec![0; 2048];

        socket.send_to(&write_buf, broadcast_address);

    //     r := make(chan Device)

    //     go func(s *net.UDPConn, request interface{}) {
    //         for {
    //             buff := make(Vec<u8>, 2048)
    //             rlen, addr, err := s.ReadFromUDP(buff)
    //             if err != nil {
    //                 break
    //             }
    //             go c.discovered(r, addr, rlen, buff)
    //         }
    //     }(sock, request)

    //     eJSON, err := json.Marshal(&request)
    //     if err != nil {
    //         return
    //     }

    //     var encrypted = c.encrypt(eJSON)
    //     _, err = sock.WriteToUDP(encrypted, broadcastAddr)
    //     if err != nil {
    //         return
    //     }
    //     started := time.Now()
    // Q:
    //     for {
    //         select {
    //         case x := <-r:

    //             info := DeviceInfoResponse{}
    //             json.Unmarshal(x.Data, &info)

    //             x.Info = &info

    //             devices = append(devices, x)
    //         default:
    //             if now := time.Now(); now.Sub(started) >= time.Duration(timeout)*time.Second {
    //                 break Q
    //             }
    //         }
    //     }

        Ok(devices)
    }

    // sendCommand is based on https://github.com/jaedle/golang-tplink-hs100
    fn send_command(&self, address: String, port: i64, command: Vec<u8>, timeout_seconds: u64) -> Result<Vec<u8>, Box<dyn Error>> {
        conn, err := net.DialTimeout("tcp", address+":"+strconv.Itoa(port), time.Duration(timeoutSeconds)*time.Second)
        if err != nil {
            return nil, err
        }
        defer conn.Close()

        writer := bufio.NewWriter(conn)
        _, err = writer.Write(c.encryptWithHeader(command))
        if err != nil {
            return nil, err
        }
        writer.Flush()

        response, err := c.readHeader(conn)
        if err != nil {
            return nil, err
        }

        payload, err := c.readPayload(conn, c.payloadLength(response))
        if err != nil {
            return nil, err
        }

        return c.decrypt(payload), nil
    }

    fn read_header(&self, conn net.Conn) -> Result<Vec<u8>, Box<dyn Error>> {
        headerReader := io.LimitReader(conn, int64(HEADER_LENGTH))
        let response = make(Vec<u8>, HEADER_LENGTH)
        _, err := headerReader.Read(response)
        return response, err
    }

    fn read_payload(&self, conn net.Conn, length uint32) -> Result<Vec<u8>, Box<dyn Error>> {
        payloadReader := io.LimitReader(conn, int64(length))
        let payload = make(Vec<u8>, length)
        _, err := payloadReader.Read(payload)
        return payload, err
    }

    fn payload_length(&self, header: Vec<u8>) -> u32 {
        payloadLength := binary.BigEndian.Uint32(header)
        return payloadLength
    }

    const HEADER_LENGTH: u32 = 4;

    fn encrypt(&self, input: Vec<u8>) -> Vec<u8> {
        s := string(input)

        key := byte(0xAB)
        b := make(Vec<u8>, len(s))
        for i := 0; i < len(s); i++ {
            b[i] = s[i] ^ key
            key = b[i]
        }
        return b
    }

    fn encrypt_with_header(&self, input: Vec<u8>) -> Vec<u8> {
        s := string(input)

        lengthPayload := len(s)
        b := make(Vec<u8>, HEADER_LENGTH+lengthPayload)
        copy(b[:HEADER_LENGTH], c.header(lengthPayload))
        copy(b[HEADER_LENGTH:], c.encrypt(input))
        return b
    }

    fn header(&self, length_payload: i64) -> Vec<u8> {
        h := make(Vec<u8>, HEADER_LENGTH)
        binary.BigEndian.PutUint32(h, uint32(lengthPayload))
        return h
    }

    fn decrypt(&self, b: Vec<u8>) -> Vec<u8> {
        k := byte(0xAB)
        var newKey byte
        for i := 0; i < len(b); i++ {
            newKey = b[i]
            b[i] = b[i] ^ k
            k = newKey
        }

        return b
    }

    fn decrypt_with_header(&self, b: Vec<u8>) -> Vec<u8> {
        return c.decrypt(c.payload(b))
    }

    fn payload(&self, b: Vec<u8>) -> Vec<u8> {
        return b[HEADER_LENGTH:]
    }

    fn discovered(r chan Device, addr *net.UDPAddr, rlen int, buff Vec<u8>) {
        r <- Device{
            Addr: addr,
            Data: c.decrypt(buff[:rlen]),
        }
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
                    if current_sample.metric_type == MetricType::Counter && current_sample.value/last_sample.value > 1.1 {
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

#[derive(Serialize, Deserialize, Debug)]
struct SystemInfo  {
    #[serde(rename = "active_mode")]
	mode :           String  ,
    #[serde(rename = "alias")]
	alias :          String  ,
    #[serde(rename = "dev_name")]
	product :        String  ,
    #[serde(rename = "device_id")]
	device_id :       String  ,
    #[serde(rename = "err_code")]
	error_code :      i32     ,
    #[serde(rename = "feature")]
	features   :     String  ,
    #[serde(rename = "fwId")]
	firmware_id  :    String  ,
    #[serde(rename = "hwId")]
	hardware_id   :   String  ,
    #[serde(rename = "hw_ver")]
	hardware_version: String  ,
    #[serde(rename = "icon_hash")]
	icon_hash :       String , 
    #[serde(rename = "latitude")]
	gps_latitude:     f32 ,
    #[serde(rename = "longitude")]
	gps_longitude:    f32 ,
    #[serde(rename = "led_off")]
	led_off   :       u8   ,
    #[serde(rename = "mac")]
	mac       :      String  ,
    #[serde(rename = "model")]
	model      :     String  ,
    #[serde(rename = "oemId")]
	oem_id       :    String  ,
    #[serde(rename = "on_time")]
	on_time       :   u32  ,
    #[serde(rename = "relay_state")]
	relay_on       :  u8   ,
    #[serde(rename = "rssi")]
	rssi           : i32     ,
    #[serde(rename = "sw_ver")]
	software_version: String  ,
    #[serde(rename = "type")]
	product_type  :   String  ,
    #[serde(rename = "updating")]
	updating      :  u8   ,
}

#[derive(Serialize, Deserialize, Debug)]
struct System {
    #[serde(rename = "get_sysinfo")]
	info:  SystemInfo,
}

#[derive(Serialize, Deserialize, Debug)]
struct RealTimeEnergy {
    #[serde(rename = "err_code")]
	error_code:          u8,
    #[serde(rename = "power_mw")]
	power_milli_watt:     f64,
    #[serde(rename = "voltage_mv")]
	voltage_milli_volt:   f64,
    #[serde(rename = "current_ma")]
	current_milli_ampere: f64,
    #[serde(rename = "total_wh")]
	total_watt_hour:      f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct EMeter {
    #[serde(rename = "get_realtime")]
	real_time: RealTimeEnergy,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeviceInfoRequest {
    #[serde(rename = "system")]
	system: System,
    #[serde(rename = "emeter")]
	e_meter: EMeter, 
}

#[derive(Serialize, Deserialize, Debug)]
struct DeviceInfoResponse {
    #[serde(rename = "system")]
	system: Option<System>,
    #[serde(rename = "emeter")]
	e_meter: Option<EMeter>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Device {
	addr: SocketAddr,
	data: Vec<u8>,
	info: Option<DeviceInfoResponse>,
}
