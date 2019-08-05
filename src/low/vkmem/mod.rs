pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;
use ash::vk::DeviceMemory;

use crate::low::vkstate::VulkanState;

pub struct VkMem<'a> {
    pub size: u64,
    pub index: u32,
    pub mem: DeviceMemory,
    state: &'a VulkanState,
}

/// For the moment, I am going to assume that 1 MemAlloc = 1 Buffer.
/// This should be changed to allow several buffer in one allocation which is more efficient.
pub struct VkBuffer<'a> {
    pub size: u64,
    pub offset: u64,
    pub buffer: vk::Buffer,
    mem: &'a VkMem<'a>,
    state: &'a VulkanState,
}

impl<'a> VkBuffer<'a> {
    pub fn new(vkstate: &'a VulkanState, vkmem: &'a VkMem, size: u64) -> Self {
        let queue_indices = &[vkstate.queue_family_index];
        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(queue_indices);

        let buffer = unsafe {
            vkstate
                .device
                .create_buffer(&buffer_create_info, None)
                .unwrap()
        };

        VkBuffer {
            size,
            offset: 0,
            buffer,
            mem: vkmem,
            state: vkstate,
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.state
                .device
                .bind_buffer_memory(self.buffer, self.mem.mem, 0)
                .expect("[ERR] Could not bind buffer memory")
        };
    }
}

// let mem_requirement = unsafe { vulkan.device.get_buffer_memory_requirements(buffer) };

impl<'a> Drop for VkBuffer<'a> {
    fn drop(&mut self) {
        unsafe {
            self.state.device.destroy_buffer(self.buffer, None);
        }
    }
}

impl<'a> VkMem<'a> {
    pub fn find_mem(vkstate: &'a VulkanState, size: u64) -> Option<Self> {
        let mem_props = unsafe {
            vkstate
                .instance
                .get_physical_device_memory_properties(vkstate.physical_device)
        };
        let mut mem_index: Option<u32> = None;
        for i in 0..mem_props.memory_type_count {
            let mem_type_props = mem_props.memory_types[i as usize];
            let buffer_max_size = mem_props.memory_heaps[mem_type_props.heap_index as usize].size;
            println!(
                "[NFO] Mem {} max heap size: {} Mio",
                i,
                buffer_max_size as f64 / 1024.0 / 1024.0
            );
            if mem_type_props
                .property_flags
                .contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
                && mem_type_props
                    .property_flags
                    .contains(vk::MemoryPropertyFlags::HOST_COHERENT)
                && mem_props.memory_heaps[mem_type_props.heap_index as usize].size > size
            {
                mem_index = Some(i);
            }
        }

        if mem_index.is_none() {
            return None;
        }

        let mem_index = mem_index.unwrap();
        let allocate_nfo = vk::MemoryAllocateInfo::builder()
            .allocation_size(size)
            .memory_type_index(mem_index)
            .build();
        let vulkan_mem = unsafe {
            vkstate
                .device
                .allocate_memory(&allocate_nfo, None)
                .expect("[ERR] Could not allocate memory in device.")
        };

        let mem_struct: VkMem<'a> = VkMem {
            size,
            index: mem_index,
            mem: vulkan_mem,
            state: vkstate,
        };
        Some(mem_struct)
    }

    pub fn map_memory<T>(&self, data: &Vec<T>, offset: u64) {
        let size = (data.len() * std::mem::size_of::<T>()) as u64;
        let buffer: *mut T = unsafe {
            self.state
                .device
                .map_memory(self.mem, offset, size, vk::MemoryMapFlags::empty())
                .expect("[ERR] Could not map memory.") as *mut T
        };

        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), buffer, data.len());
        }

        unsafe {
            self.state.device.unmap_memory(self.mem);
        }
    }

    pub fn get_memory<T>(&self, capacity: usize, offset: u64) -> Vec<T> {
        let mut output: Vec<T> = Vec::with_capacity(capacity);
        let size = (capacity * std::mem::size_of::<T>()) as u64;
        let buffer: *mut T = unsafe {
            self.state
                .device
                .map_memory(self.mem, offset, size, vk::MemoryMapFlags::empty())
                .expect("[ERR] Could not map memory.") as *mut T
        };

        unsafe {
            std::ptr::copy_nonoverlapping(buffer, output.as_mut_ptr(), capacity);
            output.set_len(capacity);
        }

        unsafe {
            self.state.device.unmap_memory(self.mem);
        }
        output
    }
}

impl<'a> Drop for VkMem<'a> {
    fn drop(&mut self) {
        unsafe {
            self.state.device.free_memory(self.mem, None);
        }
    }
}
