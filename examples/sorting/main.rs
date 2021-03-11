extern crate wyzoid;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use wyzoid::utils::to_csv;
use wyzoid::{high, utils};

/**
 * Implement a simple local bitonic merge sort. 
*/
fn main() {
    let DATA_LEN = 32;
    let input: Vec<f32> = utils::rand_vec::<f32>(DATA_LEN, 0.0, 128.0);
    let output: Vec<f32> = vec![0.0; DATA_LEN];

    let shader = PathBuf::from("examples/shaders/bin/examples/bitonic1.cs.spirv");

    let vulkan = Rc::new(wyzoid::low::vkstate::init_vulkan());

    let mut job = high::job::JobBuilder::new()
        .add_buffer(&input, 0, 0)
        .add_buffer(&output, 0, 1)
        .add_shader(&shader)
        .add_dispatch((DATA_LEN as u32, 1, 1))
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
