use crate::low::vkshader::VkShader;
use crate::low::vkstate::VulkanState;

use crate::ash::version::DeviceV1_0;
use ash::vk;
use std::rc::Rc;

pub struct VkComputePipeline {
    pub pipeline: vk::Pipeline,
    state: Rc<VulkanState>,
}

impl VkComputePipeline {
    pub fn new(state: Rc<VulkanState>, shader: &VkShader) -> Self {
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

impl Drop for VkComputePipeline {
    fn drop(&mut self) {
        unsafe {
            self.state.device.destroy_pipeline(self.pipeline, None);
        }
    }
}
