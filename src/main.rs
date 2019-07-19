extern crate csv;
extern crate vulkano;

use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;

use std::io;

fn main() {
    println!("Hello, world!");
    init_vulkan();
}

struct VulkanState<'a> {
    instance: Instance,
    physical: PhysicalDevice<'a>,
}

fn print_tick(val: bool) {
    if val {
        println!("✅");
    } else {
        println!("❌");
    }
}

fn print_queue_info(queue: vulkano::instance::QueueFamily) {
    println!("\tQueue ID: {}", queue.id());
    println!("\tQueue count: {}", queue.queues_count());

    print!("Support compute: ");
    print_tick(queue.supports_compute());

    print!("Support graphics: ");
    print_tick(queue.supports_graphics());

    print!("Explicitly support transfer: ");
    print_tick(queue.explicitly_supports_transfers());
}

fn init_vulkan() {
    let instance = Instance::new(None, &InstanceExtensions::none(), None).unwrap();
    let mut did = 0;
    for ph_dev in PhysicalDevice::enumerate(&instance) {
        println!("{} -- {}", did, ph_dev.name());
        println!("Vulkan version supported: {}", ph_dev.api_version());
        println!("Queues:");
        for queue in ph_dev.queue_families() {
            print_queue_info(queue);
        }
        did += 1;
    }

    println!("Choose physical device:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let phid: usize = input.trim().parse::<usize>().unwrap();

    let physical = PhysicalDevice::from_index(&instance, phid).unwrap();
    println!("Using devide {}.", physical.name());
}
