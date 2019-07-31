pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk::{self, PhysicalDevice};
use ash::{Device, Entry, Instance};

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
