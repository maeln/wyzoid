extern crate wyzoid;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use wyzoid::utils::to_csv;
use wyzoid::{high, utils};

fn main() {
    let DATA_LEN = 64 * 2;
    // We generate 64 random float between 0.0 and 1.0.
    let input: Vec<i32> = utils::rand_vec::<i32>(DATA_LEN, 0, 5);

    // We use a simple shader that multiply our input by two.
    let shader = PathBuf::from("examples/shaders/bin/examples/shared.cs.spirv");

    let vulkan = Rc::new(wyzoid::low::vkstate::init_vulkan());

    // We create the compute job.
    // Since our shader has a local work size of 64, we divide the number of data by 64 for the dispatch.
    let mut job = high::job::JobBuilder::new()
        .add_buffer(&input, 0, 0)
        .add_shader(&shader)
        .add_dispatch(((DATA_LEN / 64) as u32, 1, 1))
        .build(vulkan);

    job.execute();
    while job.status() == wyzoid::high::job::JobStatus::EXECUTING {
        job.wait_until_idle(1 * 1000 * 1000 * 1000);
    }
    let shader_output = job.get_output().unwrap();
    let timings = job.get_timing();
    println!("Timings:\n{}", timings);

    println!("Write to file out.csv.");
    let mut csv = String::new();
    csv.push_str(&to_csv("input", &input));
    for i in 0..shader_output.len() {
        csv.push_str(&to_csv(&format!("output {}", i), &shader_output[i]))
    }
    fs::write("out.csv", csv).expect("could not write file out.csv");
}
