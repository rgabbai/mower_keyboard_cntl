use rclrust::{qos::QoSProfile};
use rclrust_msg::std_msgs::msg::{Float64MultiArray, MultiArrayLayout};
use anyhow::Result;
use std::sync::{Arc, Mutex};
//use device_query::{DeviceQuery, DeviceState, Keycode};
use std::time::{Duration};
use tokio::time::sleep;
use std::io::{self, Write};

use termion::input::TermRead;
use termion::event::Key;
//use termion::raw::IntoRawMode;


//const DEBOUNCE_RATE: u64 = 500; // [ms] Adjust debounce time as needed to overcome keypress toggle

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the ROS2 client library
    println!("mower_keyb_cnt node activated");
    let ctx = rclrust::init()?;
    let node = ctx.create_node("mower_keyb_cnt")?;

    let publisher = node.create_publisher::<Float64MultiArray>("/joint_whacker/commands", &QoSProfile::default())?;

    let data = Arc::new(Mutex::new(vec![0.0])); // Initial array values
    let data_clone = data.clone();

    // Spawn a new thread for keyboard handling
    std::thread::spawn(move || {
        let stdin = io::stdin();
        //let mut stdout = io::stdout().into_raw_mode().unwrap(); // support raw terminal - not sure I want this
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char(' ') => {
                    let mut data = data_clone.lock().unwrap();
                    *data = data.iter().map(|&x| if x == 0.0 { 1.0 } else { 0.0 }).collect::<Vec<_>>();
                    print!("\rValue:{:?}",data[0]);
                    io::stdout().flush().unwrap();
                },
                Key::Char('q') => {
                    println!("Exit");
                    break;
                }
                _ => {}
            }
        }
    });

    loop {
        {
            let data = data.lock().unwrap();
            let msg = Float64MultiArray { 
                layout: MultiArrayLayout::default(), // Add this line
                data: data.clone(),
            };
            publisher.publish(&msg)?;
        }
        sleep(Duration::from_millis(500)).await;
    }
}
