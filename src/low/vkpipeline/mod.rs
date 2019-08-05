use crate::low::vkshader::VkShader;
use crate::low::vkstate::VulkanState;

use crate::ash::version::DeviceV1_0;
use ash::vk;

pub struct VkComputePipeline<'a> {
    pub pipeline: vk::Pipeline,
    state: &'a VulkanState,
}

impl<'a> VkComputePipeline<'a> {
    pub fn new(state: &'a VulkanState, shader: &'a VkShader<'a>) -> Self {
        let stage_create_info = vk::PipelineShaderStageCreateInfo::builder()
            .module(shader.module)
            .stage(vk::ShaderStageFlags::COMPUTE)
            .name(&shader.entry_point);

        let compute_create_info = vk::ComputePipelineCreateInfo::builder()
            .stage(stage_create_info.build())
            .layout(shader.pipeline.unwrap());

        let create_infos = &[compute_create_info.build()];
        let compute_pipeline = unsafe {
            state
                .device
                .create_compute_pipelines(vk::PipelineCache::null(), create_infos, None)
                .expect("[ERR] Could not create compute pipeline")[0]
        };

        VkComputePipeline {
            pipeline: compute_pipeline,
            state: state,
        }
    }
}

impl<'a> Drop for VkComputePipeline<'a> {
    fn drop(&mut self) {
        unsafe {
            self.state.device.destroy_pipeline(self.pipeline, None);
        }
    }
}
