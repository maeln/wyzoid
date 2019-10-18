use crate::low::{vkcmd, vkdescriptor, vkfence, vkmem, vkpipeline, vkshader, vkstate};
use crate::utils::get_fract_s;
pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;
use std::cell::RefCell;
use std::ffi::CString;
use std::fmt;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

pub struct JobTimings {
    pub upload: Duration,
    pub shader: Duration,
    pub cmd: Duration,
    pub execution: Duration,
    pub download: Duration,
}

#[derive(Debug, Clone, Copy)]
pub struct JobTimingsBuilder {
    upload_timer: Option<Instant>,
    upload: Option<Duration>,
    shader_timer: Option<Instant>,
    shader: Option<Duration>,
    cmd_timer: Option<Instant>,
    cmd: Option<Duration>,
    execution_timer: Option<Instant>,
    execution: Option<Duration>,
    download_timer: Option<Instant>,
    download: Option<Duration>,
}

impl JobTimingsBuilder {
    pub fn new() -> JobTimingsBuilder {
        JobTimingsBuilder {
            upload_timer: None,
            upload: None,
            shader_timer: None,
            shader: None,
            cmd_timer: None,
            cmd: None,
            execution_timer: None,
            execution: None,
            download_timer: None,
            download: None,
        }
    }

    pub fn start_upload(mut self) -> JobTimingsBuilder {
        self.upload_timer = Some(Instant::now());
        self
    }

    pub fn stop_upload(mut self) -> JobTimingsBuilder {
        self.upload = self.upload_timer.map(|instant| instant.elapsed());
        self
    }

    pub fn start_shader(mut self) -> JobTimingsBuilder {
        self.shader_timer = Some(Instant::now());
        self
    }

    pub fn stop_shader(mut self) -> JobTimingsBuilder {
        self.shader = self.shader_timer.map(|instant| instant.elapsed());
        self
    }

    pub fn start_cmd(mut self) -> JobTimingsBuilder {
        self.cmd_timer = Some(Instant::now());
        self
    }

    pub fn stop_cmd(mut self) -> JobTimingsBuilder {
        self.cmd = self.cmd_timer.map(|instant| instant.elapsed());
        self
    }

    pub fn start_execution(mut self) -> JobTimingsBuilder {
        self.execution_timer = Some(Instant::now());
        self
    }

    pub fn stop_execution(mut self) -> JobTimingsBuilder {
        self.execution = self.execution_timer.map(|instant| instant.elapsed());
        self
    }

    pub fn start_download(mut self) -> JobTimingsBuilder {
        self.download_timer = Some(Instant::now());
        self
    }

    pub fn stop_download(mut self) -> JobTimingsBuilder {
        self.download = self.download_timer.map(|instant| instant.elapsed());
        self
    }

    pub fn build(self) -> JobTimings {
        JobTimings {
            upload: self.upload.unwrap_or_default(),
            shader: self.shader.unwrap_or_default(),
            cmd: self.cmd.unwrap_or_default(),
            execution: self.execution.unwrap_or_default(),
            download: self.download.unwrap_or_default(),
        }
    }
}

impl fmt::Display for JobTimings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "upload: {}ms\n", get_fract_s(self.upload))?;
        write!(f, "shader: {}ms\n", get_fract_s(self.shader))?;
        write!(f, "command: {}ms\n", get_fract_s(self.cmd))?;
        write!(f, "execution: {}ms\n", get_fract_s(self.execution))?;
        write!(f, "download: {}ms\n", get_fract_s(self.download))?;
        write!(
            f,
            "total: {}ms\n",
            get_fract_s(self.upload + self.shader + self.cmd + self.execution + self.download)
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BindPoint {
    pub set: u32,
    pub bind: u32,
}

impl BindPoint {
    pub fn new(set: u32, bind: u32) -> BindPoint {
        BindPoint { set, bind }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum JobStatus {
    INIT,
    EXECUTING,
    SUCESS,
    FAILURE,
}

pub struct Job<'a, T> {
    inputs: Vec<(BindPoint, &'a Vec<T>)>,
    buffers: Vec<(BindPoint, usize)>,
    shaders: Vec<&'a PathBuf>,
    dispatch: Vec<(u32, u32, u32)>,
    state: JobState,
}

pub struct JobState {
    timing: JobTimingsBuilder,
    fence: Option<vkfence::VkFence>,
    memory: Option<vkmem::VkMem>,
    buffers: Vec<vkmem::VkBuffer>,
    vulkan: Rc<vkstate::VulkanState>,
}

pub struct JobBuilder<'a, T> {
    inputs: Vec<(BindPoint, &'a Vec<T>)>,
    buffers: Vec<(BindPoint, usize)>,
    shaders: Vec<&'a PathBuf>,
    dispatch: Vec<(u32, u32, u32)>,
}

impl<'a, T> JobBuilder<'a, T> {
    pub fn new() -> JobBuilder<'a, T> {
        JobBuilder {
            inputs: Vec::new(),
            buffers: Vec::new(),
            shaders: Vec::new(),
            dispatch: Vec::new(),
        }
    }

    pub fn add_buffer(mut self, data: &'a Vec<T>, set: u32, bind: u32) -> JobBuilder<'a, T> {
        self.inputs.push((BindPoint::new(set, bind), data));
        self
    }

    pub fn add_ro_buffer(mut self, size: usize, set: u32, bind: u32) -> JobBuilder<'a, T> {
        self.buffers.push((BindPoint::new(set, bind), size));
        self
    }

    pub fn add_shader(mut self, shader: &'a PathBuf) -> JobBuilder<'a, T> {
        self.shaders.push(shader);
        self
    }

    pub fn add_dispatch(mut self, dispatch: (u32, u32, u32)) -> JobBuilder<'a, T> {
        self.dispatch.push(dispatch);
        self
    }

    pub fn build(self, vulkan: Rc<vkstate::VulkanState>) -> Job<'a, T> {
        let state = JobState {
            fence: None,
            buffers: Vec::new(),
            memory: None,
            timing: JobTimingsBuilder::new(),
            vulkan: vulkan,
        };
        Job {
            inputs: self.inputs,
            buffers: self.buffers,
            shaders: self.shaders,
            dispatch: self.dispatch,
            state: state,
        }
    }
}

// TODO: Correctly manage set binding.
// For the moment, all binding will use the set 0 neverminding the actual value in BindPoint
impl<'a, T> Job<'a, T> {
    pub fn execute(&mut self) {
        let inputs = &self.inputs;
        let ro_buffers = &self.buffers;
        let shaders = &self.shaders;
        let dispatch = &self.dispatch;

        // Memory init.
        self.state.timing = self.state.timing.start_upload();
        let mut buffer_sizes: Vec<u64> = inputs
            .iter()
            .map(|v| (v.1.len() * std::mem::size_of::<T>()) as u64)
            .collect();
        for s in ro_buffers {
            buffer_sizes.push((s.1 * std::mem::size_of::<T>()) as u64);
        }

        self.state.buffers = buffer_sizes
            .iter()
            .map(|size| vkmem::VkBuffer::new(self.state.vulkan.clone(), *size))
            .collect();
        let (mem_size, offsets) =
            vkmem::compute_non_overlapping_buffer_alignment(&self.state.buffers);
        self.state.memory = Some(
            vkmem::VkMem::find_mem(self.state.vulkan.clone(), mem_size)
                .expect("[ERR] Could not find a memory type fitting our need."),
        );

        for i in 0..self.state.buffers.len() {
            let mbuf = self.state.buffers.get_mut(i).unwrap();
            mbuf.bind(self.state.memory.as_ref().unwrap().mem, offsets[i]);
            if i < inputs.len() {
                self.state
                    .memory
                    .as_ref()
                    .unwrap()
                    .map_buffer(inputs[i].1, mbuf);
            }
        }

        self.state.timing = self.state.timing.stop_upload();

        // Shaders
        self.state.timing = self.state.timing.start_shader();
        let mut shad_vec: Vec<Rc<RefCell<vkshader::VkShader>>> = Vec::with_capacity(shaders.len());
        let mut shad_pip_vec: Vec<vkpipeline::VkComputePipeline> =
            Vec::with_capacity(shaders.len());
        let mut shad_pipeline_layout: Vec<vk::PipelineLayout> = Vec::with_capacity(shaders.len());
        let mut shad_desc_vec: Vec<vkdescriptor::VkDescriptor> = Vec::with_capacity(shaders.len());
        let mut shad_desc_set: Vec<vkdescriptor::VkWriteDescriptor> =
            Vec::with_capacity(shaders.len());
        for path in shaders {
            shad_vec.push(Rc::new(RefCell::new(vkshader::VkShader::new(
                self.state.vulkan.clone(),
                path,
                CString::new("main").unwrap(),
            ))));
        }
        for shader in shad_vec.iter_mut() {
            for i in 0..self.state.buffers.len() {
                shader.borrow_mut().add_layout_binding(
                    i as u32,
                    1,
                    vk::DescriptorType::STORAGE_BUFFER,
                    vk::ShaderStageFlags::COMPUTE,
                );
            }
            shader.borrow_mut().create_pipeline_layout();
            shad_pipeline_layout.push(shader.borrow().pipeline.unwrap());
            shad_pip_vec.push(vkpipeline::VkComputePipeline::new(
                self.state.vulkan.clone(),
                &shader.borrow(),
            ));
            shad_desc_vec.push(vkdescriptor::VkDescriptor::new(
                self.state.vulkan.clone(),
                shader.clone(),
            ));
            shad_desc_set.push(vkdescriptor::VkWriteDescriptor::new(
                self.state.vulkan.clone(),
            ));
        }

        for descriptor in shad_desc_vec.iter_mut() {
            descriptor.add_pool_size(
                self.state.buffers.len() as u32,
                vk::DescriptorType::STORAGE_BUFFER,
            );
            descriptor.create_pool(1);
            descriptor.create_set();
        }

        let mut n = 0;
        for write_descriptor_set in shad_desc_set.iter_mut() {
            let desc_set: vk::DescriptorSet = *shad_desc_vec[n].get_first_set().unwrap();
            let mut buffers_nfos: Vec<Vec<vk::DescriptorBufferInfo>> = Vec::new();
            for i in 0..self.state.buffers.len() {
                write_descriptor_set.add_buffer(
                    self.state.buffers[i].buffer,
                    0,
                    self.state.buffers[i].size,
                );
                buffers_nfos.push(vec![write_descriptor_set.buffer_descriptors[i]]);
                write_descriptor_set.add_write_descriptors(
                    desc_set,
                    vk::DescriptorType::STORAGE_BUFFER,
                    &buffers_nfos[i],
                    i as u32,
                    0,
                );
            }
            write_descriptor_set.update_descriptors_sets();
            n += 1;
        }

        self.state.timing = self.state.timing.stop_shader();

        // Command buffers
        self.state.timing = self.state.timing.start_cmd();
        let mut cmd_buffers: Vec<usize> = Vec::with_capacity(shaders.len());
        let mut cmd_pool = vkcmd::VkCmdPool::new(self.state.vulkan.clone());

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
            for buffer in &self.state.buffers {
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
                self.state.vulkan.device.cmd_pipeline_barrier(
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
        self.state.timing = self.state.timing.stop_cmd();

        // Execution
        self.state.fence = Some(vkfence::VkFence::new(self.state.vulkan.clone(), false));
        self.state.timing = self.state.timing.start_execution();
        let queue = unsafe {
            self.state
                .vulkan
                .device
                .get_device_queue(self.state.vulkan.queue_family_index, 0)
        };
        cmd_pool.submit(queue, Some(self.state.fence.as_ref().unwrap().fence));
    }

    pub fn status(&self) -> JobStatus {
        if self.state.fence.is_none() {
            return JobStatus::INIT;
        }

        let r = self.state.fence.as_ref().map(|fence| fence.status());
        match r.unwrap() {
            vkfence::FenceStates::SIGNALED => JobStatus::SUCESS,
            vkfence::FenceStates::UNSIGNALED => JobStatus::EXECUTING,
            _ => JobStatus::FAILURE,
        }
    }

    pub fn get_output(&self) -> Option<Vec<Vec<T>>> {
        if self.status() != JobStatus::SUCESS {
            return None;
        }

        let output: Vec<Vec<T>> = self
            .state
            .buffers
            .iter()
            .map(|buf| self.state.memory.as_ref().unwrap().get_buffer(buf))
            .collect();

        Some(output)
    }

    pub fn get_timing(&self) -> JobTimings {
        self.state.timing.build()
    }

    pub fn wait_until_idle(&self, timeout: u64) -> JobStatus {
        let current_status = self.status();
        if current_status != JobStatus::EXECUTING {
            return current_status;
        }

        self.state.fence.as_ref().map(|fence| fence.wait(timeout));
        self.status()
    }
}
