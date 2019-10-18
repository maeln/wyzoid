extern crate wyzoid;
use std::path::PathBuf;
use std::rc::Rc;
use wyzoid::{high, utils};

const DATA_LEN: usize = 64;

fn main() {
    // We generate 64 random float between 0.0 and 1.0.
    let input1 = utils::rand_vec::<f32>(DATA_LEN, 0.0, 1.0);
    let input2 = utils::rand_vec::<f32>(DATA_LEN, 0.0, 1.0);

    // First shader compute sin on first buffer and cos on second buffer.
    // Second shader sub 1.0 to first buffer and add 1.0 to second buffer.
    let taylor = PathBuf::from("examples/shaders/bin/examples/taylor.cs.spirv");
    let add_sub = PathBuf::from("examples/shaders/bin/examples/add_sub.cs.spirv");

    let vulkan = Rc::new(wyzoid::low::vkstate::init_vulkan());

    // We create the compute job.
    // Since our shader has a local work size of 64, we divide the number of data by 64 for the dispatch.
    let mut job = high::job::JobBuilder::new()
        .add_buffer(&input1, 0, 0)
        .add_buffer(&input2, 0, 1)
        .add_shader(&taylor)
        .add_shader(&add_sub)
        .add_dispatch(((DATA_LEN / 64) as u32, 1, 1))
        .add_dispatch(((DATA_LEN / 64) as u32, 1, 1))
        .build(vulkan);

    job.execute();
    while job.status() == wyzoid::high::job::JobStatus::EXECUTING {
        job.wait_until_idle(1 * 1000 * 1000 * 1000);
    }
    let shader_output = job.get_output().unwrap();
    let timings = job.get_timing();

    for i in 0..DATA_LEN {
        println!(
            "[{}] in1: {}, out1: {}, in2: {}, out2: {}",
            i, input1[i], shader_output[0][i], input2[i], shader_output[1][i]
        );
    }

    println!("Timings:\n{}", timings);
}
