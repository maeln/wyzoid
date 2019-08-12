use crate::low::{vkcmd, vkdescriptor, vkmem, vkpipeline, vkshader, vkstate};
use crate::utils::get_fract_s;
pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;
use std::ffi::CString;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

pub struct Timings {
    pub init: Duration,
    pub upload: Duration,
    pub shader: Duration,
    pub cmd: Duration,
    pub execution: Duration,
    pub download: Duration,
}

impl fmt::Display for Timings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "init: {}ms\n", get_fract_s(self.init))?;
        write!(f, "upload: {}ms\n", get_fract_s(self.upload))?;
        write!(f, "shader: {}ms\n", get_fract_s(self.shader))?;
        write!(f, "command: {}ms\n", get_fract_s(self.cmd))?;
        write!(f, "execution: {}ms\n", get_fract_s(self.execution))?;
        write!(f, "download: {}ms\n", get_fract_s(self.download))?;
        write!(
            f,
            "total: {}ms\n",
            get_fract_s(
                self.init + self.upload + self.shader + self.cmd + self.execution + self.download
            )
        )
    }
}

pub struct Job<'a, T> {
    inputs: Vec<&'a Vec<T>>,
    shaders: Vec<&'a PathBuf>,
    dispatch: Vec<(u32, u32, u32)>,
}

pub struct JobBuilder<'a, T> {
    inputs: Option<Vec<&'a Vec<T>>>,
    shaders: Option<Vec<&'a PathBuf>>,
    dispatch: Option<Vec<(u32, u32, u32)>>,
}

impl<'a, T> JobBuilder<'a, T> {
    pub fn new() -> JobBuilder<'a, T> {
        JobBuilder {
            inputs: Some(Vec::new()),
            shaders: Some(Vec::new()),
            dispatch: Some(Vec::new()),
        }
    }

    pub fn add_buffer(mut self, data: &'a Vec<T>) -> JobBuilder<'a, T> {
        self.inputs.as_mut().and_then(|b| {
            b.push(data);
            Some(b)
        });
        self
    }

    pub fn add_shader(mut self, shader: &'a PathBuf) -> JobBuilder<'a, T> {
        self.shaders.as_mut().and_then(|s| {
            s.push(shader);
            Some(s)
        });
        self
    }

    pub fn add_dispatch(mut self, dispatch: (u32, u32, u32)) -> JobBuilder<'a, T> {
        self.dispatch.as_mut().and_then(|d| {
            d.push(dispatch);
            Some(d)
        });
        self
    }

    pub fn build(self) -> Job<'a, T> {
        let inputs = self.inputs.unwrap();
        let shaders = self.shaders.unwrap();
        let dispatch = self.dispatch.unwrap();

        Job {
            inputs: inputs,
            shaders: shaders,
            dispatch: dispatch,
        }
    }
}

impl<'a, T> Job<'a, T> {
    pub fn execute(&self) -> Vec<Vec<T>> {
        let inputs = &self.inputs;
        let shaders = &self.shaders;
        let dispatch = &self.dispatch;

        // Vulkan init.
        let vulkan = vkstate::init_vulkan();
        vkstate::print_work_limits(&vulkan);

        // Memory init.
        let buffer_sizes: Vec<u64> = inputs
            .iter()
            .map(|v| (v.len() * std::mem::size_of::<T>()) as u64)
            .collect();
        let mut buffers: Vec<vkmem::VkBuffer> = buffer_sizes
            .iter()
            .map(|size| vkmem::VkBuffer::new(&vulkan, *size))
            .collect();
        let (mem_size, offsets) = vkmem::compute_non_overlapping_buffer_alignment(&buffers);

        let vk_mem = vkmem::VkMem::find_mem(&vulkan, mem_size);
        if vk_mem.is_none() {
            panic!("[ERR] Could not find a memory type fitting our need.");
        }

        let vk_mem = vk_mem.unwrap();
        for i in 0..buffers.len() {
            let mbuf = buffers.get_mut(i).unwrap();
            mbuf.bind(vk_mem.mem, offsets[i]);
            vk_mem.map_buffer(inputs[i], mbuf);
        }

        let mut shad_vec: Vec<vkshader::VkShader> = Vec::with_capacity(shaders.len());
        let mut shad_pip_vec: Vec<vkpipeline::VkComputePipeline> =
            Vec::with_capacity(shaders.len());
        let mut shad_pipeline_layout: Vec<vk::PipelineLayout> = Vec::with_capacity(shaders.len());
        let mut shad_desc_vec: Vec<vkdescriptor::VkDescriptor> = Vec::with_capacity(shaders.len());
        let mut shad_desc_set: Vec<vkdescriptor::VkWriteDescriptor> =
            Vec::with_capacity(shaders.len());
        for path in shaders {
            shad_vec.push(vkshader::VkShader::new(
                &vulkan,
                path,
                CString::new("main").unwrap(),
            ));
        }

        for shader in shad_vec.iter_mut() {
            shader.add_layout_binding(
                0,
                1,
                vk::DescriptorType::STORAGE_BUFFER,
                vk::ShaderStageFlags::COMPUTE,
            );
            shader.add_layout_binding(
                1,
                1,
                vk::DescriptorType::STORAGE_BUFFER,
                vk::ShaderStageFlags::COMPUTE,
            );
            shader.create_pipeline_layout();
            shad_pipeline_layout.push(shader.pipeline.unwrap());
            shad_pip_vec.push(vkpipeline::VkComputePipeline::new(&vulkan, shader));
            shad_desc_vec.push(vkdescriptor::VkDescriptor::new(&vulkan, shader));
            shad_desc_set.push(vkdescriptor::VkWriteDescriptor::new(&vulkan));
        }

        for descriptor in shad_desc_vec.iter_mut() {
            descriptor.add_pool_size(2, vk::DescriptorType::STORAGE_BUFFER);
            descriptor.create_pool(1);
            descriptor.create_set();
        }

        let mut n = 0;
        for write_descriptor_set in shad_desc_set.iter_mut() {
            for buffer in &buffers {
                write_descriptor_set.add_buffer(buffer.buffer, 0, buffer.size);
            }
            let desc_set: vk::DescriptorSet = *shad_desc_vec[n].get_first_set().unwrap();
            let mut bf1 = Vec::new();
            let mut bf2 = Vec::new();
            bf1.push(write_descriptor_set.buffer_descriptors[0]);
            bf2.push(write_descriptor_set.buffer_descriptors[1]);
            write_descriptor_set.add_write_descriptors(
                desc_set,
                vk::DescriptorType::STORAGE_BUFFER,
                &bf1,
                0,
                0,
            );

            write_descriptor_set.add_write_descriptors(
                desc_set,
                vk::DescriptorType::STORAGE_BUFFER,
                &bf2,
                1,
                0,
            );
            write_descriptor_set.update_descriptors_sets();
            n += 1;
        }

        let mut cmd_buffers: Vec<usize> = Vec::with_capacity(shaders.len());
        let mut cmd_pool = vkcmd::VkCmdPool::new(&vulkan);

        for _ in 0..shaders.len() {
            cmd_buffers.push(cmd_pool.create_cmd_buffer(vk::CommandBufferLevel::PRIMARY));
        }

        for i in cmd_buffers {
            cmd_pool.begin_cmd(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT, i);
            cmd_pool.bind_pipeline(shad_pip_vec[i].pipeline, vk::PipelineBindPoint::COMPUTE, i);
            cmd_pool.bind_descriptor(
                shad_pipeline_layout[i],
                vk::PipelineBindPoint::COMPUTE,
                &shad_desc_vec[i].set,
                i,
            );

            let d = dispatch[i];
            cmd_pool.dispatch(d.0, d.1, d.2, i);

            // Memory barrier
            let mut buffer_barrier: Vec<vk::BufferMemoryBarrier> = Vec::new();
            for buffer in &buffers {
                buffer_barrier.push(
                    vk::BufferMemoryBarrier::builder()
                        .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                        .dst_access_mask(vk::AccessFlags::SHADER_READ)
                        .buffer(buffer.buffer)
                        .size(vk::WHOLE_SIZE)
                        .build(),
                );
            }
            unsafe {
                vulkan.device.cmd_pipeline_barrier(
                    cmd_pool.cmd_buffers[i],
                    vk::PipelineStageFlags::COMPUTE_SHADER,
                    vk::PipelineStageFlags::COMPUTE_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &buffer_barrier,
                    &[],
                );
            }

            cmd_pool.end_cmd(i);
        }

        let queue = unsafe { vulkan.device.get_device_queue(vulkan.queue_family_index, 0) };
        cmd_pool.submit(queue);

        unsafe {
            vulkan
                .device
                .queue_wait_idle(queue)
                .expect("[ERR] Error while waiting for queue to be idle.")
        };

        // Download results.
        let output: Vec<Vec<T>> = buffers.iter().map(|buf| vk_mem.get_buffer(buf)).collect();

        output
    }
}
