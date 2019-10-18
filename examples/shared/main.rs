extern crate wyzoid;
use std::path::PathBuf;
use std::rc::Rc;
use wyzoid::{high, utils};

fn main() {
    let DATA_LEN = 64 * 2;
    // We generate 64 random float between 0.0 and 1.0.
    let input: Vec<f32> = utils::rand_vec::<f32>(DATA_LEN, 0.0, 1.0);
    let mut output: Vec<f32> = Vec::with_capacity(DATA_LEN);
    for _ in 0..DATA_LEN {
        output.push(0.0);
    }
    let mut comp = input.clone();
    let start = std::time::Instant::now();
    comp.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("cpu: {}ms", wyzoid::utils::get_fract_s(start.elapsed()));

    // We use a simple shader that multiply our input by two.
    let shader = PathBuf::from("examples/shaders/bin/examples/oddeven.cs.spirv");

    let vulkan = Rc::new(wyzoid::low::vkstate::init_vulkan());

    // We create the compute job.
    // Since our shader has a local work size of 64, we divide the number of data by 64 for the dispatch.
    let mut job = high::job::JobBuilder::new()
        .add_buffer(&input, 0, 0)
        .add_buffer(&output, 0, 1)
        .add_shader(&shader)
        .add_dispatch(((DATA_LEN / 64) as u32, 1, 1))
        .build(vulkan);

    job.execute();
    while job.status() == wyzoid::high::job::JobStatus::EXECUTING {
        job.wait_until_idle(1 * 1000 * 1000 * 1000);
    }
    let shader_output = job.get_output().unwrap();
    let timings = job.get_timing();

    for i in 0..DATA_LEN {
        if !wyzoid::utils::f32_cmp(shader_output[1][i], comp[i], 0.0001) {
            println!("Diff: {} {} {}", i, shader_output[0][i], comp[i]);
        }
    }

    println!("Timings:\n{}", timings);
}
