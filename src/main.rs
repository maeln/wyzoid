extern crate ash;
extern crate csv;

use std::convert::From;
use std::ffi::{CStr, CString};
use std::fs;
use std::io::{self, BufRead};
use std::mem;
use std::os::raw::{c_char, c_void};
use std::path::PathBuf;

use ash::extensions::ext::DebugReport;
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
    let buffer = unsafe {
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

    let buffer_create_info = vk::BufferCreateInfo::builder()
        .size(buffer_size)
        .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .queue_family_indices(&[vulkan.queue_family_index])
        .build();

    let buffer = unsafe {
        vulkan
            .device
            .create_buffer(&buffer_create_info, None)
            .unwrap()
    };

    unsafe {
        vulkan
            .device
            .bind_buffer_memory(buffer, vulkan_mem, buffer_size);
    }

    let shader_bytecode = to_vec32(
        load_file(&PathBuf::from("shaders/bin/double/double.cs.spriv"))
            .expect("[ERR] Could not load shader file."),
    );

    let shader_module_create_info = vk::ShaderModuleCreateInfo::builder()
        .code(&shader_bytecode)
        .build();
    let shader_module = unsafe {
        vulkan
            .device
            .create_shader_module(&shader_module_create_info, None)
            .expect("[ERR] Could not create shader module.")
    };

    let descriptor_layout_binding_info = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::COMPUTE)
        .build();
    let descriptor_layout_create_info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(&[descriptor_layout_binding_info])
        .build();

    let descriptor_layout = unsafe {
        vulkan
            .device
            .create_descriptor_set_layout(&descriptor_layout_create_info, None)
            .expect("[ERR] Could not create Descriptor Layout.")
    };

    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(&[descriptor_layout])
        .build();
    let pipeline_layout = unsafe {
        vulkan
            .device
            .create_pipeline_layout(&pipeline_layout_create_info, None)
            .expect("[ERR] Could not create Pipeline Layout")
    };

    let entry_point = CString::new("main").unwrap();
    let stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
        .module(shader_module)
        .stage(vk::ShaderStageFlags::COMPUTE)
        .name(&entry_point)
        .build();
    let compute_create_info = vk::ComputePipelineCreateInfo::builder()
        .stage(stage_create_info)
        .layout(pipeline_layout)
        .build();

    let compute_pipeline = unsafe {
        vulkan
            .device
            .create_compute_pipelines(vk::PipelineCache::null(), &[compute_create_info], None)
            .expect("[ERR] Could not create compute pipeline")[0]
    };

    let descriptor_pool_size = vk::DescriptorPoolSize::builder()
        .descriptor_count(1)
        .ty(vk::DescriptorType::STORAGE_BUFFER)
        .build();
    let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder()
        .max_sets(1)
        .pool_sizes(&[descriptor_pool_size])
        .build();

    let descriptor_pool = unsafe {
        vulkan
            .device
            .create_descriptor_pool(&descriptor_pool_create_info, None)
            .expect("[ERR] Could not create descriptor pool.")
    };

    let descriptor_allocate = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(descriptor_pool)
        .set_layouts(&[descriptor_layout])
        .build();

    let descriptor_set = unsafe {
        vulkan
            .device
            .allocate_descriptor_sets(&descriptor_allocate)
            .expect("[ERR] Could not create descriptor set.")[0]
    };

    let descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
        .buffer(buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE)
        .build();
    let write_descriptor_set = vk::WriteDescriptorSet::builder()
        .dst_set(descriptor_set)
        .dst_binding(0)
        .dst_array_element(0)
        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
        .buffer_info(&[descriptor_buffer_info])
        .build();

    unsafe {
        vulkan
            .device
            .update_descriptor_sets(&[write_descriptor_set], &[])
    };

    let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
        .queue_family_index(vulkan.queue_family_index)
        .build();
    let command_pool = unsafe {
        vulkan
            .device
            .create_command_pool(&command_pool_create_info, None)
            .expect("[ERR] Could not create command pool.")
    };

    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .command_buffer_count(1)
        .level(vk::CommandBufferLevel::PRIMARY)
        .build();
    let command_buffer = unsafe {
        vulkan
            .device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("[ERR] Could not create command buffer")[0]
    };

    let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
        .build();
    unsafe {
        vulkan
            .device
            .begin_command_buffer(command_buffer, &command_buffer_begin_info)
            .expect("[ERR] Could not begin command buffer.")
    };

    unsafe {
        vulkan.device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            compute_pipeline,
        )
    };

    unsafe {
        vulkan.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            pipeline_layout,
            0,
            &[descriptor_set],
            &[],
        )
    };

    unsafe {
        vulkan
            .device
            .cmd_dispatch(command_buffer, buffer_capacity as u32, 1, 1);
    };

    unsafe {
        vulkan
            .device
            .end_command_buffer(command_buffer)
            .expect("[ERR] Could not end command buffer.");
    };

    let queue = unsafe { vulkan.device.get_device_queue(vulkan.queue_family_index, 0) };
    let submit_info = vk::SubmitInfo::builder()
        .command_buffers(&[command_buffer])
        .build();

    unsafe {
        vulkan
            .device
            .queue_submit(queue, &[submit_info], vk::Fence::null())
            .expect("[ERR] Could not submit queue.")
    };

    unsafe {
        vulkan
            .device
            .queue_wait_idle(queue)
            .expect("[ERR] Error while waiting for queue to be idle.")
    };

    unsafe {
        let out_buffer = vulkan
            .device
            .map_memory(vulkan_mem, 0, buffer_size, mem_map_flags)
            .expect("[ERR] Could not map memory at output.");
        let output: Vec<f32> = Vec::from_raw_parts(
            out_buffer as *mut f32,
            buffer_capacity as usize,
            buffer_capacity as usize,
        );
        for i in 0..output.len() {
            print!("{} ", output[i]);
        }
        mem::forget(output);
    }
}

fn to_vec32(vecin: Vec<u8>) -> Vec<u32> {
    unsafe { vecin.align_to::<u32>().1.to_vec() }
}

struct VulkanState {
    instance: Instance,
    physical_device: PhysicalDevice,
    device: Device,
    queue: Queue,
    queue_family_index: u32,
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
    let string = unsafe { CString::from_raw(cstr.as_mut_ptr()) };
    mem::forget(cstr);
    String::from(string.to_string_lossy())
}

fn extension_names() -> Vec<*const i8> {
    vec![DebugReport::name().as_ptr()]
}

unsafe extern "system" fn vulkan_debug_callback(
    _: vk::DebugReportFlagsEXT,
    _: vk::DebugReportObjectTypeEXT,
    _: u64,
    _: usize,
    _: i32,
    _: *const c_char,
    p_message: *const c_char,
    _: *mut c_void,
) -> u32 {
    println!("{:?}", CStr::from_ptr(p_message));
    vk::FALSE
}

fn ash_vulkan() -> VulkanState {
    let layer_names = [CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()];
    let layers_names_raw: Vec<*const i8> = layer_names
        .iter()
        .map(|raw_name| raw_name.as_ptr())
        .collect();
    let extension_names_raw = extension_names();

    let entry = Entry::new().unwrap();
    let app_info = vk::ApplicationInfo {
        api_version: ash::vk_make_version!(1, 0, 0),
        p_application_name: "Wyzoid".as_ptr() as *const i8,
        application_version: ash::vk_make_version!(1, 0, 0),
        ..Default::default()
    };

    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layers_names_raw)
        .enabled_extension_names(&extension_names_raw);

    let instance: Instance = unsafe { entry.create_instance(&create_info, None).unwrap() };

    let debug_info = vk::DebugReportCallbackCreateInfoEXT::builder()
        .flags(
            vk::DebugReportFlagsEXT::ERROR
                | vk::DebugReportFlagsEXT::WARNING
                | vk::DebugReportFlagsEXT::PERFORMANCE_WARNING
                | vk::DebugReportFlagsEXT::DEBUG
                | vk::DebugReportFlagsEXT::INFORMATION,
        )
        .pfn_callback(Some(vulkan_debug_callback));

    let debug_report_loader = DebugReport::new(&entry, &instance);
    let debug_call_back = unsafe {
        debug_report_loader
            .create_debug_report_callback(&debug_info, None)
            .unwrap()
    };

    let physical: PhysicalDevice;
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
        queue_family_index: queue_index,
    }
}
