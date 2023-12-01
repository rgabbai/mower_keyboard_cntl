use rclrust::{qos::QoSProfile};
use rclrust_msg::std_msgs::msg::{Float64MultiArray, MultiArrayLayout};
use anyhow::Result;
use std::sync::{Arc, Mutex};
use device_query::{DeviceQuery, DeviceState, Keycode};
use std::time::{Duration,Instant};
use tokio::time::sleep;
use std::io::{self, Write};


const DEBOUNCE_RATE: u64 = 500; // [ms] Adjust debounce time as needed to overcome keypress toggle

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
        let device_state = DeviceState::new();
        let mut last_space_press = Instant::now();
        let debounce_duration = Duration::from_millis(DEBOUNCE_RATE); // Adjust debounce time as needed
    
        loop {
            let keys = device_state.get_keys();
            if keys.contains(&Keycode::Space) {
                let now = Instant::now();
                if now.duration_since(last_space_press) >= debounce_duration {
                    let mut data = data_clone.lock().unwrap();
                    *data = data.iter().map(|&x| if x == 0.0 { 1.0 } else { 0.0 }).collect::<Vec<_>>();
                    last_space_press = now;
                    print!("\rValue:{:?}",data[0]);
                    io::stdout().flush().unwrap();
                }
            }
            std::thread::sleep(Duration::from_millis(50)); // Reduce CPU usage
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
        sleep(Duration::from_millis(100)).await;
    }
}
