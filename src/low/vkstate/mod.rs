pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk::{self, PhysicalDevice};
use ash::{Device, Entry, Instance};

use crate::utils::{cstr2string, tick};

use std::ffi::{CStr, CString};
use std::io::{self, BufRead};
use std::os::raw::{c_char, c_void};

use ash::extensions::ext::DebugReport;

use log::{info, warn};

pub struct VulkanState {
    pub entry: Entry,
    pub instance: Instance,
    pub physical_device: PhysicalDevice,
    pub device: Device,
    pub queue_family_index: u32,
    pub debug_report_loader: ash::extensions::ext::DebugReport,
    pub debug_callback: vk::DebugReportCallbackEXT,
}

impl Drop for VulkanState {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            self.device.destroy_device(None);
            self.debug_report_loader
                .destroy_debug_report_callback(self.debug_callback, None);
            self.instance.destroy_instance(None);
        }
    }
}

pub fn print_work_limits(vulkan: &VulkanState) {
    let physical_device_props = unsafe {
        vulkan
            .instance
            .get_physical_device_properties(vulkan.physical_device)
    };

    let physical_limits = physical_device_props.limits;
    let work_group_count = physical_limits.max_compute_work_group_count;
    let work_group_size = physical_limits.max_compute_work_group_size;
    let work_group_invocation = physical_limits.max_compute_work_group_invocations;

    info!(
        "Device max work group count: [{}, {}, {}]",
        work_group_count[0], work_group_count[1], work_group_count[2]
    );
    info!(
        "Device max work group size: [{}, {}, {}]",
        work_group_size[0], work_group_size[1], work_group_size[2]
    );
    info!(
        "Device max work group invocation: {}",
        work_group_invocation
    );
    info!(
        "minStorageBufferOffset: {}",
        physical_limits.min_storage_buffer_offset_alignment
    );
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
    warn!("\n{:?}", CStr::from_ptr(p_message));
    vk::FALSE
}

fn extension_names() -> Vec<*const i8> {
    vec![DebugReport::name().as_ptr()]
}

pub fn init_vulkan() -> VulkanState {
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
        info!("Only one physical device ({}) defaulting to it.", phy_name);
    } else {
        // We don't use the logger here because we need user
        // feedback so we need whatever we print to be visible in all cases.
        println!("Physical device:");
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
            println!("\t* GRAPHICS: {}", tick(dev_graphics));
            println!("\t* COMPUTE: {}", tick(dev_compute));
            println!("\t* TRANSFER: {}", tick(dev_transfer));
            println!("\t* SPARSE OPS: {}", tick(dev_sparse));

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
        info!("Using device {}.", phy_name);
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

    VulkanState {
        entry,
        instance,
        physical_device: physical,
        device,
        queue_family_index: queue_index,
        debug_callback,
        debug_report_loader,
    }
}
