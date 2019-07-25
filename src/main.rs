extern crate ash;
extern crate csv;
extern crate vulkano;

use std::convert::From;
use std::ffi::CStr;
use std::fs;
use std::io::{self, BufRead};
use std::mem;
use std::path::PathBuf;
use std::str;
use std::sync::Arc;

pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk::{self, PhysicalDevice, Queue};
use ash::{Device, Entry, Instance};

fn main() {
    let vulkan = ash_vulkan();

    let buffer_capacity = 1024;
    let buffer_size: u64 = buffer_capacity * (std::mem::size_of::<f32>() as u64);
    let mem_props = unsafe {
        vulkan
            .instance
            .get_physical_device_memory_properties(vulkan.physical_device)
    };

    let mut mem_index: Option<usize> = None;
    for i in 0..(mem_props.memory_type_count as usize) {
        let mem_type_props = mem_props.memory_types[i];
        if mem_type_props
            .property_flags
            .contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
            && mem_type_props
                .property_flags
                .contains(vk::MemoryPropertyFlags::HOST_COHERENT)
            && mem_props.memory_heaps[mem_type_props.heap_index as usize].size > buffer_size
        {
            mem_index = Some(i);
            break;
        }
    }

    if mem_index.is_none() {
        panic!("[ERR] Could not find a memory type fitting our need.");
    }

    let allocate_nfo = vk::MemoryAllocateInfo::builder()
        .allocation_size(buffer_size)
        .memory_type_index(mem_index.unwrap() as u32)
        .build();
    let vulkan_mem = unsafe {
        vulkan
            .device
            .allocate_memory(&allocate_nfo, None)
            .expect("[ERR] Could not allocate memory in device.")
    };

    let mem_map_flags = vk::MemoryMapFlags::empty();
    let mut buffer = unsafe {
        vulkan
            .device
            .map_memory(vulkan_mem, 0, buffer_size, mem_map_flags)
            .expect("[ERR] Could not map memory.")
    };
    let mut vec_buff: Vec<f32> = unsafe {
        Vec::from_raw_parts(
            buffer as *mut f32,
            buffer_capacity as usize,
            buffer_capacity as usize,
        )
    };
    for i in 0..buffer_capacity {
        vec_buff[i as usize] = i as f32;
    }
    mem::forget(vec_buff);

    unsafe {
        vulkan.device.unmap_memory(vulkan_mem);
    }
}

struct VulkanState {
    instance: Instance,
    physical_device: PhysicalDevice,
    device: Device,
    queue: Queue,
}

fn load_file(file: &PathBuf) -> Option<Vec<u8>> {
    let contents = fs::read(file);
    match contents {
        Ok(file_str) => Some(file_str),
        Err(err) => {
            eprintln!("[ERR] Impossible to read file {} : {}", file.display(), err);

            None
        }
    }
}

fn print_tick(val: bool) {
    if val {
        println!("✅");
    } else {
        println!("❌");
    }
}

fn cstr2string(mut cstr: Vec<i8>) -> String {
    let name_len = cstr.len();
    let name_cap = cstr.capacity();
    let string =
        unsafe { String::from_raw_parts(cstr.as_mut_ptr() as *mut u8, name_len, name_cap) };
    mem::forget(cstr);
    string
}

fn ash_vulkan() -> VulkanState {
    let entry = Entry::new().unwrap();
    let app_info = vk::ApplicationInfo {
        api_version: ash::vk_make_version!(1, 0, 0),
        p_application_name: "Wyzoid".as_ptr() as *const i8,
        application_version: ash::vk_make_version!(1, 0, 0),
        ..Default::default()
    };

    let create_info = vk::InstanceCreateInfo {
        p_application_info: &app_info,
        ..Default::default()
    };

    let instance: Instance = unsafe { entry.create_instance(&create_info, None).unwrap() };

    let mut physical: PhysicalDevice;
    let phy_count = unsafe { instance.enumerate_physical_devices().unwrap() };
    if phy_count.len() == 1 {
        physical = phy_count[0];
        let properties = unsafe { instance.get_physical_device_properties(physical) };

        let phy_name = cstr2string(properties.device_name.to_vec());
        println!(
            "[NFO] Only one physical device ({}) defaulting to it.",
            phy_name
        );
    } else {
        println!("[NFO] Physical device:");
        let mut i = 0;
        for dev in phy_count.clone() {
            let properties = unsafe { instance.get_physical_device_properties(dev) };
            let dev_name = cstr2string(properties.device_name.to_vec());
            let (mut dev_graphics, mut dev_compute, mut dev_transfer, mut dev_sparse) =
                (false, false, false, false);
            unsafe {
                instance
                    .get_physical_device_queue_family_properties(dev)
                    .iter()
                    .for_each(|nfo| {
                        if !dev_graphics && nfo.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                            dev_graphics = true;
                        }
                        if !dev_compute && nfo.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                            dev_compute = true;
                        }
                        if !dev_transfer && nfo.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                            dev_transfer = true;
                        }
                        if !dev_sparse && nfo.queue_flags.contains(vk::QueueFlags::SPARSE_BINDING) {
                            dev_sparse = true;
                        }
                    });
            }

            println!("- [{}] {}:", i, dev_name);
            print!("\t* GRAPHICS: ");
            print_tick(dev_graphics);
            print!("\t* COMPUTE: ");
            print_tick(dev_compute);
            print!("\t* TRANSFER: ");
            print_tick(dev_transfer);
            print!("\t* SPARSE OPS: ");
            print_tick(dev_sparse);

            i += 1;
        }

        println!("Use: ");
        let mut line = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut line).unwrap();
        let phy_id = line
            .trim()
            .parse::<usize>()
            .expect("[ERR] Please write a number.");
        physical = phy_count[phy_id];
        let properties = unsafe { instance.get_physical_device_properties(physical) };
        let phy_name = cstr2string(properties.device_name.to_vec());
        println!("[NFO] Using device {}.", phy_name);
    }

    // Get queue family:
    let queue_index = unsafe {
        instance
            .get_physical_device_queue_family_properties(physical)
            .iter()
            .enumerate()
            .filter_map(|(index, ref nfo)| {
                let support_compute = nfo.queue_flags.contains(vk::QueueFlags::COMPUTE);
                let support_transfer = nfo.queue_flags.contains(vk::QueueFlags::TRANSFER);
                match support_compute && support_transfer {
                    true => Some(index),
                    false => None,
                }
            })
            .nth(0)
            .expect("[ERR] Could not find a valid queue.") as u32
    };

    let features = vk::PhysicalDeviceFeatures {
        ..Default::default()
    };

    let queue_create_info = [vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_index)
        .queue_priorities(&[1.0])
        .build()];

    let device_create_info_builder = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_info)
        .enabled_features(&features)
        .enabled_extension_names(&[]);
    let device: Device = unsafe {
        instance
            .create_device(physical, &device_create_info_builder, None)
            .unwrap()
    };

    let device_queue: Queue = unsafe { device.get_device_queue(queue_index, 0) };

    VulkanState {
        instance,
        physical_device: physical,
        device,
        queue: device_queue,
    }
}
