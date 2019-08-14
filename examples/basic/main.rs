extern crate wyzoid;
use std::path::PathBuf;
use wyzoid::{high, utils};

const DATA_LEN: usize = 64;

fn main() {
    // We generate 64 random float between 0.0 and 1.0.
    let input = utils::rand_vec::<f32>(DATA_LEN, 0.0, 1.0);

    // We use a simple shader that multiply our input by two.
    let shader = PathBuf::from("examples/shaders/bin/examples/double.cs.spirv");

    // We create the compute job.
    // Since our shader has a local work size of 64, we divide the number of data by 64 for the dispatch.
    let job = high::job::JobBuilder::new()
        .add_buffer(&input)
        .add_shader(&shader)
        .add_dispatch(((DATA_LEN / 64) as u32, 1, 1))
        .build();

    let (shader_output, timings) = job.execute();

    for i in 0..DATA_LEN {
        println!("[{}] in: {}, out: {}", i, input[i], shader_output[0][i]);
    }

    println!("Timings:\n{}", timings);
}
