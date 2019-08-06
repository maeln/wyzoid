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

pub fn one_shot_job<T>(
    shader_path: &PathBuf,
    input: &Vec<T>,
    dispatch: (u32, u32, u32),
) -> (Vec<T>, Timings) {
    // Vulkan init.
    let init_t = Instant::now();
    let vulkan = vkstate::init_vulkan();
    let init_d = init_t.elapsed();

    // Memory init.
    let buffer_size: u64 = (input.len() * std::mem::size_of::<T>()) as u64;
    let mem_t = Instant::now();
    let vk_buffer = vkmem::VkBuffer::new(&vulkan, buffer_size);
    let vk_mem = vkmem::VkMem::find_mem(&vulkan, vk_buffer.get_buffer_memory_requirements());
    if vk_mem.is_none() {
        panic!("[ERR] Could not find a memory type fitting our need.");
    }

    let vk_mem = vk_mem.unwrap();
    vk_mem.bind(vk_buffer.buffer, 0);
    vk_mem.map_memory(input, 0);
    let mem_d = mem_t.elapsed();

    // Shader init.
    let shader_t = Instant::now();
    let mut shader = vkshader::VkShader::new(&vulkan, shader_path, CString::new("main").unwrap());

    shader.add_layout_binding(
        0,
        1,
        vk::DescriptorType::STORAGE_BUFFER,
        vk::ShaderStageFlags::COMPUTE,
    );
    shader.create_pipeline_layout();

    let compute_pipeline = vkpipeline::VkComputePipeline::new(&vulkan, &shader);

    let mut descriptor = vkdescriptor::VkDescriptor::new(&vulkan, &shader);
    descriptor.add_pool_size(1, vk::DescriptorType::STORAGE_BUFFER);
    descriptor.create_pool(1);
    descriptor.create_set();

    let mut write_descriptor_set = vkdescriptor::VkWriteDescriptor::new(&vulkan);
    write_descriptor_set.add_buffer(vk_buffer.buffer, 0, vk::WHOLE_SIZE);
    write_descriptor_set.add_write_descriptors(
        *descriptor.get_first_set().unwrap(),
        vk::DescriptorType::STORAGE_BUFFER,
        0,
        0,
    );
    write_descriptor_set.update_descriptors_sets();
    let shader_d = shader_t.elapsed();

    // Command pool & bufffer init.
    let cmd_t = Instant::now();
    let mut cmd_pool = vkcmd::VkCmdPool::new(&vulkan);
    let cmd_buf_i = cmd_pool.create_cmd_buffer(vk::CommandBufferLevel::PRIMARY);
    cmd_pool.begin_cmd(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT, cmd_buf_i);
    cmd_pool.bind_pipeline(
        compute_pipeline.pipeline,
        vk::PipelineBindPoint::COMPUTE,
        cmd_buf_i,
    );
    cmd_pool.bind_descriptor(
        shader.pipeline.unwrap(),
        vk::PipelineBindPoint::COMPUTE,
        &descriptor.set,
        cmd_buf_i,
    );

    cmd_pool.dispatch(dispatch.0, dispatch.1, dispatch.2, cmd_buf_i);
    cmd_pool.end_cmd(cmd_buf_i);
    let cmd_d = cmd_t.elapsed();

    // Start the job and wait for it to finish.
    let execute_t = Instant::now();
    let queue = unsafe { vulkan.device.get_device_queue(vulkan.queue_family_index, 0) };
    cmd_pool.submit(queue);

    unsafe {
        vulkan
            .device
            .queue_wait_idle(queue)
            .expect("[ERR] Error while waiting for queue to be idle.")
    };
    let execute_d = execute_t.elapsed();

    // Download results.
    let download_t = Instant::now();
    let shader_output = vk_mem.get_memory::<T>(input.len(), 0);
    let download_d = download_t.elapsed();

    let timings = Timings {
        init: init_d,
        upload: mem_d,
        shader: shader_d,
        cmd: cmd_d,
        execution: execute_d,
        download: download_d,
    };

    (shader_output, timings)
}
