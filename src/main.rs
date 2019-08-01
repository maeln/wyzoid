extern crate ash;
// extern crate csv;

mod utils;
mod vkdescriptor;
mod vkmem;
mod vkpipeline;
mod vkshader;
mod vkstate;

use utils::{cstr2string, get_fract_s, print_tick};

use std::convert::From;
use std::ffi::{CStr, CString};
use std::io::{self, BufRead};
use std::os::raw::{c_char, c_void};
use std::path::PathBuf;

use ash::extensions::ext::DebugReport;
pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk::{self, PhysicalDevice};
use ash::{Device, Entry, Instance};

use std::time::Instant;

fn doit(data: *mut f32) -> *mut f32 {
    let mut addr = data;
    unsafe {
        let x = addr.read();
        let y = addr.offset(1).read();
        let z = addr.offset(2).read();
        addr.write(x * 2.0);
        addr = addr.offset(1);
        addr.write(x * y);
        addr = addr.offset(1);
        addr.write(x + y);
        addr = addr.offset(1);
        addr.write(z - x);
        addr = addr.offset(1);
    }
    addr
}

const BUFFER_CAPACITY: u64 = 4096 * 4096;

fn main() {
    let vulkan = ash_vulkan();
    println!("[NFO] Vulkan initialized.");
    let mut hello: Vec<f32> = Vec::with_capacity(BUFFER_CAPACITY as usize);
    for i in 0..BUFFER_CAPACITY {
        hello.push(i as f32);
    }

    let physical_device_props = unsafe {
        vulkan
            .instance
            .get_physical_device_properties(vulkan.physical_device)
    };

    let physical_limits = physical_device_props.limits;
    let work_group_count = physical_limits.max_compute_work_group_count;
    let work_group_size = physical_limits.max_compute_work_group_size;
    let work_group_invocation = physical_limits.max_compute_work_group_invocations;

    println!(
        "[NFO] Device max work group count: [{}, {}, {}]",
        work_group_count[0], work_group_count[1], work_group_count[2]
    );
    println!(
        "[NFO] Device max work group size: [{}, {}, {}]",
        work_group_size[0], work_group_size[1], work_group_size[2]
    );
    println!(
        "[NFO] Device max work group invocation: {}",
        work_group_invocation
    );

    let buffer_size: u64 = BUFFER_CAPACITY * (std::mem::size_of::<f32>() as u64);
    let buff_start = Instant::now();

    let vk_mem = vkmem::VkMem::find_mem(&vulkan, buffer_size);

    if vk_mem.is_none() {
        panic!("[ERR] Could not find a memory type fitting our need.");
    }

    let vk_mem = vk_mem.unwrap();
    vk_mem.map_memory::<f32>(&hello, 0);

    let vk_buffer = vkmem::VkBuffer::new(&vulkan, &vk_mem, buffer_size);
    vk_buffer.bind();
    println!("[NFO] Mem: {} ms", get_fract_s(buff_start));
    let shader_stuff = Instant::now();

    let mut shader = vkshader::VkShader::new(
        &vulkan,
        &PathBuf::from("shaders/bin/double/double.cs.spriv"),
        CString::new("main").unwrap(),
    );

    shader.add_layout_binding(
        0,
        1,
        vk::DescriptorType::STORAGE_BUFFER,
        vk::ShaderStageFlags::COMPUTE,
    );
    shader.create_pipeline_layout();

    let compute_pipeline = vkpipeline::VkComputePipeline::new(&vulkan, &shader);

    let mut descriptor = vkdescriptor::VkDescriptor::new(&vulkan, &shader);
    descriptor.add_pool_size(1, vk::DescriptorType::STORAGE_BUFFER);
    descriptor.create_pool(1);
    descriptor.create_set();

    let mut write_descriptor_set = vkdescriptor::VkWriteDescriptor::new(&vulkan);
    write_descriptor_set.add_buffer(vk_buffer.buffer, 0, vk::WHOLE_SIZE);
    write_descriptor_set.add_write_descriptors(
        *descriptor.get_first_set().unwrap(),
        vk::DescriptorType::STORAGE_BUFFER,
        0,
        0,
    );
    write_descriptor_set.update_descriptors_sets();

    println!("[NFO] shd: {} ms", get_fract_s(shader_stuff));
    let cmdstuff = Instant::now();
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
        .level(vk::CommandBufferLevel::PRIMARY);
    let command_buffer = unsafe {
        vulkan
            .device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("[ERR] Could not create command buffer")[0]
    };

    let command_buffer_begin_info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

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
            compute_pipeline.pipeline,
        )
    };

    unsafe {
        vulkan.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            shader.pipeline.unwrap(),
            0,
            &[descriptor.set[0]],
            &[],
        )
    };

    unsafe {
        vulkan
            .device
            .cmd_dispatch(command_buffer, BUFFER_CAPACITY as u32 / 4 / 64, 1, 1);
    };

    unsafe {
        vulkan
            .device
            .end_command_buffer(command_buffer)
            .expect("[ERR] Could not end command buffer.");
    };

    let cmd_buffer = &[command_buffer];
    let queue = unsafe { vulkan.device.get_device_queue(vulkan.queue_family_index, 0) };
    let submit_info = vk::SubmitInfo::builder().command_buffers(cmd_buffer);
    println!("[NFO] Cmd: {} ms", get_fract_s(cmdstuff));
    let start = Instant::now();
    unsafe {
        vulkan
            .device
            .queue_submit(queue, &[submit_info.build()], vk::Fence::null())
            .expect("[ERR] Could not submit queue.")
    };

    unsafe {
        vulkan
            .device
            .queue_wait_idle(queue)
            .expect("[ERR] Error while waiting for queue to be idle.")
    };

    println!("[NFO] Time taken: {} ms", get_fract_s(start));
    let new_start = Instant::now();
    let mut buf_ptr = hello.as_mut_ptr();
    for _ in 0..(BUFFER_CAPACITY as usize / 4) {
        buf_ptr = doit(buf_ptr);
    }
    let new_spent = get_fract_s(new_start);
    println!("[NFO] Time taken: {} ms", new_spent);

    unsafe {
        let mut out_buffer: *mut f32 = vulkan
            .device
            .map_memory(vk_mem.mem, 0, buffer_size, vk::MemoryMapFlags::empty())
            .expect("[ERR] Could not map memory at output.")
            as *mut f32;
        let mut diff_count = 0;
        for i in 0..BUFFER_CAPACITY as usize {
            if !(hello[i] == *out_buffer
                || (hello[i] + 0.01 > *out_buffer && hello[i] - 0.01 < *out_buffer))
            {
                diff_count += 1;
                println!("DIFF[{}]: {} // {}", i, hello[i], *out_buffer);
            }
            if diff_count > 5 {
                break;
            }
            out_buffer = out_buffer.offset(1);
        }
    }
    print!("\n");
    // cleanup
    unsafe {
        vulkan
            .device
            .free_command_buffers(command_pool, &[command_buffer]);
        vulkan.device.destroy_command_pool(command_pool, None);
    }
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
    println!("\n[VAL] {:?}", CStr::from_ptr(p_message));
    vk::FALSE
}

fn ash_vulkan() -> vkstate::VulkanState {
    let layer_names = [CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()];
    let layers_names_raw: Vec<*const i8> = layer_names
        .iter()
        .map(|raw_name| raw_name.as_ptr())
        .collect();
    let extension_names_raw = extension_names();

    let app_name = CString::new("Wyzoid").unwrap();
    let entry = Entry::new().unwrap();
    let app_info = vk::ApplicationInfo::builder()
        .api_version(ash::vk_make_version!(1, 0, 0))
        .application_name(&app_name)
        .application_version(ash::vk_make_version!(1, 0, 0));
    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layers_names_raw)
        .enabled_extension_names(&extension_names_raw);

    let instance: Instance = unsafe { entry.create_instance(&create_info, None).unwrap() };

    let debug_info = vk::DebugReportCallbackCreateInfoEXT::builder()
        .flags(
            vk::DebugReportFlagsEXT::ERROR
                | vk::DebugReportFlagsEXT::WARNING
                | vk::DebugReportFlagsEXT::PERFORMANCE_WARNING,
        )
        .pfn_callback(Some(vulkan_debug_callback));

    let debug_report_loader = DebugReport::new(&entry, &instance);
    let debug_callback = unsafe {
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
                if support_compute && support_transfer {
                    Some(index)
                } else {
                    None
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

    vkstate::VulkanState {
        entry,
        instance,
        physical_device: physical,
        device,
        queue_family_index: queue_index,
        debug_callback,
        debug_report_loader,
    }
}
