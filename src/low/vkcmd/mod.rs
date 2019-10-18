use crate::low::vkstate::VulkanState;

use crate::ash::version::DeviceV1_0;
use ash::vk;
use std::rc::Rc;

pub struct VkCmdPool {
    pub cmd_pool: vk::CommandPool,
    pub cmd_buffers: Vec<vk::CommandBuffer>,
    state: Rc<VulkanState>,
}

impl VkCmdPool {
    pub fn new(state: Rc<VulkanState>) -> VkCmdPool {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(state.queue_family_index)
            .build();
        let command_pool = unsafe {
            state
                .device
                .create_command_pool(&command_pool_create_info, None)
                .expect("[ERR] Could not create command pool.")
        };
        VkCmdPool {
            cmd_pool: command_pool,
            cmd_buffers: Vec::new(),
            state,
        }
    }

    pub fn create_cmd_buffer(&mut self, level: vk::CommandBufferLevel) -> usize {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.cmd_pool)
            .command_buffer_count(1)
            .level(level);
        let command_buffer = unsafe {
            self.state
                .device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("[ERR] Could not create command buffer")[0]
        };
        self.cmd_buffers.push(command_buffer);
        self.cmd_buffers.len() - 1
    }

    pub fn begin_cmd(&self, usage: vk::CommandBufferUsageFlags, cmd_buffer_index: usize) {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder().flags(usage);

        unsafe {
            self.state
                .device
                .begin_command_buffer(
                    self.cmd_buffers[cmd_buffer_index],
                    &command_buffer_begin_info,
                )
                .expect("[ERR] Could not begin command buffer.")
        };
    }

    pub fn end_cmd(&self, cmd_buffer_index: usize) {
        unsafe {
            self.state
                .device
                .end_command_buffer(self.cmd_buffers[cmd_buffer_index])
                .expect("[ERR] Could not end command buffer.");
        };
    }

    pub fn bind_pipeline(
        &self,
        pipeline: vk::Pipeline,
        pipeline_type: vk::PipelineBindPoint,
        cmd_buffer_index: usize,
    ) {
        unsafe {
            self.state.device.cmd_bind_pipeline(
                self.cmd_buffers[cmd_buffer_index],
                pipeline_type,
                pipeline,
            )
        };
    }

    pub fn bind_descriptor(
        &self,
        layout: vk::PipelineLayout,
        pipeline_type: vk::PipelineBindPoint,
        descriptor_sets: &[vk::DescriptorSet],
        cmd_buffer_index: usize,
    ) {
        unsafe {
            self.state.device.cmd_bind_descriptor_sets(
                self.cmd_buffers[cmd_buffer_index],
                pipeline_type,
                layout,
                0,
                descriptor_sets,
                &[],
            )
        };
    }

    pub fn dispatch(&self, x: u32, y: u32, z: u32, cmd_buffer_index: usize) {
        unsafe {
            self.state
                .device
                .cmd_dispatch(self.cmd_buffers[cmd_buffer_index], x, y, z);
        };
    }

    pub fn submit(&self, queue: vk::Queue, fence: Option<vk::Fence>) {
        let submit_info = vk::SubmitInfo::builder().command_buffers(&self.cmd_buffers);
        unsafe {
            self.state
                .device
                .queue_submit(
                    queue,
                    &[submit_info.build()],
                    fence.unwrap_or(vk::Fence::null()),
                )
                .expect("[ERR] Could not submit queue.")
        };
    }
}

impl Drop for VkCmdPool {
    fn drop(&mut self) {
        unsafe {
            self.state
                .device
                .free_command_buffers(self.cmd_pool, &self.cmd_buffers);
            self.state.device.destroy_command_pool(self.cmd_pool, None);
        }
    }
}
