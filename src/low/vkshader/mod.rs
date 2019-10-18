use ash::version::DeviceV1_0;
use ash::vk;
use std::ffi::CString;
use std::path::PathBuf;
use std::rc::Rc;

use crate::low::vkstate::VulkanState;
use crate::utils::{load_file, to_vec32};

pub struct VkShader {
    pub bytecode: Vec<u32>,
    pub module: vk::ShaderModule,
    pub layouts_bindings: Vec<vk::DescriptorSetLayoutBinding>,
    pub layout: Vec<vk::DescriptorSetLayout>,
    pub pipeline: Option<vk::PipelineLayout>,
    pub entry_point: CString,
    state: Rc<VulkanState>,
}

impl VkShader {
    pub fn new(state: Rc<VulkanState>, path: &PathBuf, entry_point: CString) -> Self {
        let shader_bytecode = to_vec32(load_file(path).expect("[ERR] Could not load shader file."));

        let shader_module_create_info = vk::ShaderModuleCreateInfo::builder()
            .code(&shader_bytecode)
            .build();
        let shader_module = unsafe {
            state
                .device
                .create_shader_module(&shader_module_create_info, None)
                .expect("[ERR] Could not create shader module.")
        };

        VkShader {
            bytecode: shader_bytecode,
            module: shader_module,
            layouts_bindings: Vec::new(),
            layout: Vec::new(),
            pipeline: None,
            entry_point,
            state,
        }
    }

    pub fn add_layout_binding(
        &mut self,
        binding: u32,
        count: u32,
        descriptor_type: vk::DescriptorType,
        stage: vk::ShaderStageFlags,
    ) {
        let descriptor_layout_binding_info = vk::DescriptorSetLayoutBinding::builder()
            .binding(binding)
            .descriptor_type(descriptor_type)
            .descriptor_count(count)
            .stage_flags(stage);
        self.layouts_bindings
            .push(descriptor_layout_binding_info.build());
    }

    pub fn create_pipeline_layout(&mut self) {
        let descriptor_layout_create_info =
            vk::DescriptorSetLayoutCreateInfo::builder().bindings(&self.layouts_bindings);

        let descriptor_layout = unsafe {
            self.state
                .device
                .create_descriptor_set_layout(&descriptor_layout_create_info, None)
                .expect("[ERR] Could not create Descriptor Layout.")
        };

        self.layout.push(descriptor_layout);
        let pipeline_layout_create_info =
            vk::PipelineLayoutCreateInfo::builder().set_layouts(&self.layout);
        let pipeline_layout = unsafe {
            self.state
                .device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("[ERR] Could not create Pipeline Layout")
        };

        self.pipeline = Some(pipeline_layout);
    }
}

impl Drop for VkShader {
    fn drop(&mut self) {
        unsafe {
            if let Some(pipeline) = self.pipeline {
                self.state.device.destroy_pipeline_layout(pipeline, None);
            }
            for descriptor in self.layout.iter() {
                self.state
                    .device
                    .destroy_descriptor_set_layout(*descriptor, None);
            }
            self.state.device.destroy_shader_module(self.module, None);
        }
    }
}
