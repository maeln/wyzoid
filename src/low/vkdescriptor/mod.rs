use crate::low::vkshader::VkShader;
use crate::low::vkstate::VulkanState;

use crate::ash::version::DeviceV1_0;
use ash::vk;
use std::cell::RefCell;
use std::rc::Rc;

pub struct VkDescriptor {
    pub pool_sizes: Vec<vk::DescriptorPoolSize>,
    pub pool: Option<vk::DescriptorPool>,
    pub set: Vec<vk::DescriptorSet>,
    state: Rc<VulkanState>,
    shader: Rc<RefCell<VkShader>>,
}

impl VkDescriptor {
    pub fn new(state: Rc<VulkanState>, shader: Rc<RefCell<VkShader>>) -> Self {
        VkDescriptor {
            pool_sizes: Vec::new(),
            pool: None,
            set: Vec::new(),
            state,
            shader,
        }
    }

    pub fn add_pool_size(&mut self, count: u32, descriptor_type: vk::DescriptorType) {
        let descriptor_pool_size = vk::DescriptorPoolSize::builder()
            .descriptor_count(count)
            .ty(descriptor_type);
        self.pool_sizes.push(descriptor_pool_size.build());
    }

    pub fn create_pool(&mut self, max_sets: u32) {
        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(max_sets)
            .pool_sizes(&self.pool_sizes);
        let descriptor_pool = unsafe {
            self.state
                .device
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("[ERR] Could not create descriptor pool.")
        };
        self.pool = Some(descriptor_pool);
    }

    pub fn create_set(&mut self) {
        let borrowed_layout = &self.shader.borrow().layout;
        let descriptor_allocate = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.pool.unwrap())
            .set_layouts(borrowed_layout);

        let mut descriptor_set = unsafe {
            self.state
                .device
                .allocate_descriptor_sets(&descriptor_allocate)
                .expect("[ERR] Could not create descriptor set.")
        };

        self.set.append(&mut descriptor_set);
    }

    pub fn get_first_set(&self) -> Option<&vk::DescriptorSet> {
        self.set.first()
    }
}

impl Drop for VkDescriptor {
    fn drop(&mut self) {
        unsafe {
            if let Some(pool) = self.pool {
                self.state.device.destroy_descriptor_pool(pool, None);
            }
        }
    }
}

pub struct VkWriteDescriptor {
    pub buffer_descriptors: Vec<vk::DescriptorBufferInfo>,
    pub write_descriptors: Vec<vk::WriteDescriptorSet>,
    state: Rc<VulkanState>,
}

impl VkWriteDescriptor {
    pub fn new(state: Rc<VulkanState>) -> Self {
        VkWriteDescriptor {
            buffer_descriptors: Vec::new(),
            write_descriptors: Vec::new(),
            state,
        }
    }

    pub fn add_buffer(&mut self, buffer: vk::Buffer, offset: u64, range: u64) {
        let descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(buffer)
            .offset(offset)
            .range(range)
            .build();
        self.buffer_descriptors.push(descriptor_buffer_info);
    }

    pub fn add_write_descriptors(
        &mut self,
        descriptor_set: vk::DescriptorSet,
        descriptor_type: vk::DescriptorType,
        buffer_info: &[vk::DescriptorBufferInfo],
        dst_binding: u32,
        dst_array: u32,
    ) {
        let write_descriptor_set = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_binding(dst_binding)
            .dst_array_element(dst_array)
            .descriptor_type(descriptor_type)
            .buffer_info(buffer_info)
            .build();
        self.write_descriptors.push(write_descriptor_set);
    }

    pub fn update_descriptors_sets(&self) {
        unsafe {
            self.state
                .device
                .update_descriptor_sets(&self.write_descriptors, &[])
        };
    }
}
