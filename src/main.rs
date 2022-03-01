// https://github.com/szeged/blurz/blob/master/examples/test6.rs
// https://dev.to/lcsfelix/using-rust-blurz-to-read-from-a-ble-device-gmb

extern crate blurz;

#[macro_use]
extern crate lazy_static;

use log::{info, warn, debug, error};
use std::error::Error;
use std::thread;
use std::time::Duration;

use regex::Regex;

use std::str;
use std::collections::HashMap;
// use sscanf::scanf;

use blurz::bluetooth_adapter::BluetoothAdapter as Adapter;
use blurz::bluetooth_device::BluetoothDevice as Device;
use blurz::bluetooth_discovery_session::BluetoothDiscoverySession as DiscoverySession;
use blurz::bluetooth_session::BluetoothSession as Session;
use blurz::BluetoothGATTService;
use blurz::BluetoothGATTCharacteristic;

fn _stringify_hashmap(_a: HashMap<u16, Vec<u8>>) -> String {
    //todo
    String::from(":todo:")
}

macro_rules! _vec_of_strings {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

const UUID_REGEX: &str = r"([0-9a-f]{8})-(?:[0-9a-f]{4}-){3}[0-9a-f]{12}";
lazy_static! {
    static ref RE: Regex = Regex::new(UUID_REGEX).unwrap();
}

const _INTERESTING_DEVICES4: [&'static str;4] = [
    "CD:EC:27:E7:E7:B0" //Jinou_Beacon (ibeacon,eddystone) humidity & temp sensor
    ,"/org/bluez/hci0/dev_CD_EC_27_E7_E7_B0" //uuid:
    ,"D9:BE:F7:E2:C4:E6" //Blue Maestro humidity/temp sensor
    ,"/org/bluez/hci0/dev_D9_BE_F7_E2_C4_E6"
];

const INTERESTING_DEVICES2: [&'static str;2] = [
    "CD:EC:27:E7:E7:B0" //Jinou_Beacon (ibeacon,eddystone) humidity & temp sensor
    ,"/org/bluez/hci0/dev_CD_EC_27_E7_E7_B0" //uuid:
];

const INTERESTING_DEVICES: &[&str;2] = &INTERESTING_DEVICES2;

fn _print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

fn connect_device(bt_session: &Session, device: &Device) -> bool {
        debug!(">>in connect_device\n\tsession={:?}\n\tdevice={:?}\n", bt_session, device);    
        let mut connected: bool = false; 
        let mut connection_attemp_counter: i32 = 0;
        while !connected {
            debug!("Trying connecting to {}...", device.get_name().unwrap());
            connection_attemp_counter = connection_attemp_counter + 1;
            match device.disconnect() {
                Ok(_)  => {},
                Err(e) => { error!("Disconnect error {}", e);}
            };
            match device.connect(10000) {
                Ok(_) => {
                    debug!("Connect attempt done. Waiting 10s to see if connected.....");
                    thread::sleep(Duration::from_millis(10000)); //this is needed apparently
                    connected = device.is_connected().unwrap();
                },
                Err(e) => warn!("Connect attempt errored: {}", e)
            }
            if connection_attemp_counter >=8 && !connected { return false; }
            if !connected {thread::sleep(Duration::from_millis(8000));}
        }
        
        debug!("Connected?: {:?}", connected);

        const UUID_REGEX: &str = r"([0-9a-f]{8})-(?:[0-9a-f]{4}-){3}[0-9a-f]{12}";
        lazy_static! {
            static ref RE: Regex = Regex::new(UUID_REGEX).unwrap();
        }

        debug!("Getting GATT services...");
        let all_gatt_services = device.get_gatt_services().unwrap();
        debug!("Listing GATT services... {} found.", all_gatt_services.len());
        if all_gatt_services.len() == 0 { connected=false; return connected;}

        for service_path in all_gatt_services {
            let gatt_service = BluetoothGATTService::new(&bt_session, service_path);
            // println!("service:{:?}", gatt_service.get_uuid().unwrap());

            let characteristics = gatt_service.get_gatt_characteristics().unwrap();
            for characteristic_path in characteristics {
                let characteristic = BluetoothGATTCharacteristic::new(&bt_session, characteristic_path);
                // println!("\tchar:{:?}", characteristic.get_uuid().unwrap());
                // println!("\tget_value  {:?}", characteristic.get_value().unwrap());
                
                if characteristic.get_uuid().unwrap() == "0000aa21-0000-1000-8000-00805f9b34fb" {
                    match characteristic.read_value(None) {
                        Ok(value) => {
                            debug!("get_value: {:?}", value);
                            println!("{}.{}C {}.{}%RH", 
                                value[1],
                                value[2],
                                value[4],
                                value[5]
                            );
                        },
                        Err(e) => {
                            error!("Some error reading data: {}", e);
                        }
                    };

                    // println!("\tread_value {:?}", value);
                    // let input = format!("{} {} {} {}", 
                    //     value[1],
                    //     value[2],
                    //     value[4],
                    //     value[5]
                    // );
                    // let parsed = scanf!(input, "{usize:x} {usize:x} {usize:x} {usize:x}");
                    // let (ti, td, hi, hd) = parsed.unwrap();
                    // println!(">>>>>>>>>{}.{}C {}.{}%RH", ti, td, hi, hd);
                }
            }

        }

        /* TODO reduce check frequency to 30s?
        0XAA22(Read,Write)：start/stop checking temp and humid. 0x01(start),
        0x00(stop).

        0XAA23(Read,Write)：checking frequency, unit(10ms), default is 100, that is
        checking every second.
        */ 

    debug!(">>out connect_device");
    connected
}

fn show_device_properties(device: &Device) {
    debug!(">>in show_device_properties");
    debug!("alias  {:?}", device.get_alias());
    debug!("name:  {:?}", device.get_name());
    debug!("addr:  {:?}", device.get_address());
    debug!("id  :  {:?}", device.get_id());
    // debug!("rssi:  {:?}", device.get_rssi());
    
    debug!("uuids          {:?}", device.get_uuids());

    debug!(">>out show_device_properties");
}

fn btscan(_interesting_devices_map: HashMap<String, String>) -> Result<Vec<String>, Box<dyn Error>> {
    debug!(">>in btscan");
    let bt_session = &Session::create_session(None)?;
    let adapter: Adapter = Adapter::init(bt_session)?;
    debug!("BT adapter id={}", adapter.get_id());
    adapter.set_powered(true)?;
    let adapter_id = adapter.get_id();
    
    let mut seen_devices: Vec<String> = Vec::new();

    let session = DiscoverySession::create_session(
        bt_session,
        adapter_id
    )?;

    thread::sleep(Duration::from_millis(300));
    
    let mut found = false;
    let mut counter=0;
    
    debug!("Start discovery.");
    session.start_discovery()?;

    '_discovery_loop: loop {
        counter+=1;
        debug!("Waiting to see what devices come up.....");
        thread::sleep(Duration::from_millis(5000));
        let device_list = adapter.get_device_list()?;
        debug!("Devices seen: ({})\n{:?}", device_list.len(), device_list);

        'device_loop: for device_path in device_list {
            // Jinou_Sensor_HumiTemp
            if !seen_devices.iter().any(|i| i == &device_path)
            { 
                debug!("Adding to seen devices {}", device_path);
                let newdev = device_path.clone();
                seen_devices.push(newdev.to_owned());
                debug!("Seen so far {:?}", seen_devices)
            }

            if !INTERESTING_DEVICES.iter().any(|i| i == &device_path)
            { 
                debug!("Skipping {}", device_path);
                continue 'device_loop;
            }
            found = true;
            debug!("Found interesting device!!! {}", device_path);

            let device = &Device::new(&bt_session, device_path.clone());
            debug!(">>>Device: {:?} Name: {:?}", device_path, device.get_name().ok());

            show_device_properties(device);

            //get gett profile
            match connect_device(bt_session, device) {
                false => warn!("Failed to connect to {}", device.get_name()?),
                true => {
                    //get services
                    // get_services(device);
                    //get characteristics
                    // get_characteristics(service);
    
                    debug!("Disconnecting...");
                    if let Err(e) = device.disconnect() {
                        warn!("  Error on disconnect: {:?}", e);
                    }
                    else {
                        debug!("Disconnected OK.");
                    }
                }
            }
            adapter.remove_device(device.get_id())?;
        }
        
        if !found {
            warn!("Didn't find anything I was looking for... retry...")
        }

        if counter > 10 || found {break;}
    } 
    session.stop_discovery()?;
    debug!("Ended session.");
    debug!(">>out btscan");
    Ok(seen_devices)
}

//RUST_LOG=DEBUG cargo run
fn main() {
    debug!(">>in main");
    debug!("No output? Run: RUST_LOG=DEBUG cargo run");
    env_logger::init();

    // TODO  provide a ble device name to look for
    let _interesting_devices_map: HashMap<String, String> = HashMap::new();    

    match btscan(_interesting_devices_map) {
        // TODO stuff
        Ok(devices_spotted) => {
            '_device_loop: for device_path in devices_spotted {
                info!("Device seen: {}", device_path);
            }
        }, 
        Err(e) => error!("{:?}", e),
    }
    debug!(">>out main");
}
