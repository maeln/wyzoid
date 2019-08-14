extern crate wyzoid;
use std::path::PathBuf;
use wyzoid::{high, utils};

const DATA_LEN: usize = 64;

fn main() {
    // We generate 64 random float between 0.0 and 1.0.
    let input1 = utils::rand_vec::<f32>(DATA_LEN, 0.0, 1.0);
    let input2 = utils::rand_vec::<f32>(DATA_LEN, 0.0, 1.0);

    // We use a shader that compute sinus and cosinus using taylor series.
    // Buffer one will be sinus, buffer two will be cosinus.
    let shader = PathBuf::from("examples/shaders/bin/examples/taylor.cs.spirv");

    // We create the compute job.
    // Since our shader has a local work size of 64, we divide the number of data by 64 for the dispatch.
    let job = high::job::JobBuilder::new()
        .add_buffer(&input1)
        .add_buffer(&input2)
        .add_shader(&shader)
        .add_dispatch(((DATA_LEN / 64) as u32, 1, 1))
        .build();

    let (shader_output, timings) = job.execute();

    for i in 0..DATA_LEN {
        println!("[{}] in1: {}, sin_cpu: {}, sin_gpu: {}, in2: {}, cos_cpu: {}, cos_gpu: {}", 
        i, input1[i], f32::sin(input1[i]), shader_output[0][i], input2[i], f32::cos(input2[i]), shader_output[1][i]);
    }

    println!("Timings:\n{}", timings);
}
