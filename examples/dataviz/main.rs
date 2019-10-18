extern crate wyzoid;
use std::path::PathBuf;
use std::rc::Rc;

use wyzoid::{high, utils};

fn main() {
    // Our first shader generate FBM noise.
    // The second one color the noise using Turbo colormap from Google AI.
    let fbm = PathBuf::from("examples/shaders/bin/examples/fbm.cs.spirv");
    let turbo = PathBuf::from("examples/shaders/bin/examples/turbo.cs.spirv");

    let vulkan = Rc::new(wyzoid::low::vkstate::init_vulkan());

    // We create the compute job.
    // The first shader has a local size of (8,8), so we need to dispatch (32,32) job
    // fill our 256x256 image.
    // The second one use a local size of 64 linearly over x, so we juste need to dispatch
    // the size of the image divided by 64 to cover the entire space.
    let mut job = high::job::JobBuilder::new()
        .add_ro_buffer(256 * 256, 0, 0)
        .add_ro_buffer(256 * 256 * 4, 0, 1)
        .add_shader(&fbm)
        .add_shader(&turbo)
        .add_dispatch((32, 32, 1))
        .add_dispatch((256 * 256 / 64, 1, 1))
        .build(vulkan);

    job.execute();
    while job.status() == wyzoid::high::job::JobStatus::EXECUTING {
        job.wait_until_idle(1 * 1000 * 1000 * 1000);
    }
    let shader_output = job.get_output().unwrap();
    let timings = job.get_timing();

    println!("Timings:\n{}", timings);
    // We oversize the img data vec but whatever
    let mut img_data: Vec<f32> = Vec::with_capacity(shader_output[1].len());
    // [vec4] -> [vec3]
    for i in 0..(shader_output[1].len() / 4) {
        img_data.push(shader_output[1][i * 4 + 0]);
        img_data.push(shader_output[1][i * 4 + 1]);
        img_data.push(shader_output[1][i * 4 + 2]);
    }

    let ppm = utils::to_ppm(&img_data, 256, 256);
    std::fs::write(PathBuf::from("./ex.ppm"), ppm.unwrap()).expect("Could not write image.");
}
