extern crate ash;
// extern crate csv;

mod utils;
mod vkcmd;
mod vkdescriptor;
mod vkmem;
mod vkpipeline;
mod vkshader;
mod vkstate;

pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;
use std::ffi::CString;
use std::path::PathBuf;
use std::time::Instant;
use utils::get_fract_s;

fn process(x: f32, y: f32, z: f32, w: f32, id: f32) -> (f32, f32, f32, f32) {
    let mut o: (f32, f32, f32, f32) = (0.0, 0.0, 0.0, 0.0);
    for i in 0..64 {
        let f = i as f32;
        let b = (x * f, y * f, z * f, w * f);
        let t = id + 1.0;
        let p = (b.0 / t, b.1 / t, b.2 / t, b.3 / t);
        let v = f32::sin(f / 100.0);
        o = (o.0 + p.0, o.1 + p.1, o.2 + p.2, o.3 + p.3);
        o = (o.0 * v, o.1 * v, o.2 * v, o.3 * v);
    }

    o
}

fn doit(data: *mut f32, id: f32) -> *mut f32 {
    let mut addr = data;
    unsafe {
        let x = addr.read();
        let y = addr.offset(1).read();
        let z = addr.offset(2).read();
        let w = addr.offset(3).read();
        let res = process(x, y, z, w, id);
        addr.write(res.0);
        addr = addr.offset(1);
        addr.write(res.1);
        addr = addr.offset(1);
        addr.write(res.2);
        addr = addr.offset(1);
        addr.write(res.3);
        addr = addr.offset(1);
    }
    addr
}

const BUFFER_CAPACITY: u64 = 4096 * 4096;

fn main() {
    let vulkan = vkstate::init_vulkan();
    println!("[NFO] Vulkan initialized.");
    let mut hello: Vec<f32> = Vec::with_capacity(BUFFER_CAPACITY as usize);
    for i in 0..BUFFER_CAPACITY {
        hello.push(i as f32);
    }

    vkstate::print_work_limits(&vulkan);

    let buffer_size: u64 = BUFFER_CAPACITY * (std::mem::size_of::<f32>() as u64);
    let buff_start = Instant::now();

    let vk_mem = vkmem::VkMem::find_mem(&vulkan, buffer_size);

    if vk_mem.is_none() {
        panic!("[ERR] Could not find a memory type fitting our need.");
    }

    let vk_mem = vk_mem.unwrap();
    vk_mem.map_memory::<f32>(&hello, 0);

    let vk_buffer = vkmem::VkBuffer::new(&vulkan, &vk_mem, buffer_size);
    vk_buffer.bind();
    println!("[NFO] Mem: {} ms", get_fract_s(buff_start));
    let shader_stuff = Instant::now();

    let mut shader = vkshader::VkShader::new(
        &vulkan,
        &PathBuf::from("shaders/bin/double/double.cs.spriv"),
        CString::new("main").unwrap(),
    );

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

    println!("[NFO] shd: {} ms", get_fract_s(shader_stuff));
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

    cmd_pool.dispatch(BUFFER_CAPACITY as u32 / 4 / 64, 1, 1, cmd_buf_i);
    cmd_pool.end_cmd(cmd_buf_i);

    let queue = unsafe { vulkan.device.get_device_queue(vulkan.queue_family_index, 0) };
    let start = Instant::now();
    cmd_pool.submit(queue);

    unsafe {
        vulkan
            .device
            .queue_wait_idle(queue)
            .expect("[ERR] Error while waiting for queue to be idle.")
    };

    println!("[NFO] Time taken: {} ms", get_fract_s(start));
    let new_start = Instant::now();
    let mut buf_ptr = hello.as_mut_ptr();
    for i in 0..(BUFFER_CAPACITY as usize / 4) {
        buf_ptr = doit(buf_ptr, i as f32);
    }
    let new_spent = get_fract_s(new_start);
    println!("[NFO] Time taken: {} ms", new_spent);
    let shader_output = vk_mem.get_memory(BUFFER_CAPACITY as usize, 0);
    let mut diff_count = 0;
    for i in 0..BUFFER_CAPACITY as usize {
        if !utils::f32_cmp(hello[i], shader_output[i], 0.001) {
            diff_count += 1;
            println!("DIFF[{}]: {} // {}", i, hello[i], shader_output[i]);
        }
        if diff_count > 5 {
            break;
        }
    }
}
