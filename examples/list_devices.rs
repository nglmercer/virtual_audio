use cpal::traits::{DeviceTrait, HostTrait};

fn main() {
    let host = cpal::default_host();
    println!("Default Host: {:?}", host.id());

    println!("Input devices:");
    for device in host.input_devices().unwrap() {
        println!("  {}", device.name().unwrap());
    }

    println!("Output devices:");
    for device in host.output_devices().unwrap() {
        println!("  {}", device.name().unwrap());
    }
}
