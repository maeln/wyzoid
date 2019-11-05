extern crate wyzoid;
use std::os::raw::c_void;
use std::path::PathBuf;
use std::rc::Rc;
use wyzoid::{high, utils};

const DATA_LEN: usize = 64;

struct MyUbo {
    my_value: f32,
}

impl MyUbo {
    fn new(val: f32) -> MyUbo {
        MyUbo { my_value: val }
    }
}

impl wyzoid::low::vkmem::Serializable for MyUbo {
    fn serialize(&self) -> *const c_void {
        let ptr = &self.my_value as *const f32;
        ptr as *const c_void
    }

    fn byte_size(&self) -> usize {
        std::mem::size_of::<f32>()
    }
}

fn main() {
    // We generate 64 random float between 0.0 and 1.0.
    let input = utils::rand_vec::<f32>(DATA_LEN, 0.0, 1.0);

    // We use a simple shader that multiply our input by two.
    let shader = PathBuf::from("examples/shaders/bin/examples/multiply.cs.spirv");

    let vulkan = Rc::new(wyzoid::low::vkstate::init_vulkan());

    // We create the compute job.
    // Since our shader has a local work size of 64, we divide the number of data by 64 for the dispatch.
    let ubo_val = MyUbo::new(4.0);
    // Yeah we need to come up with a proper solution.

    let mut job = high::job::JobBuilder::new()
        .add_buffer(&input, 0, 0)
        .add_ubo(&ubo_val, 0, 1)
        .add_shader(&shader)
        .add_dispatch(((DATA_LEN / 64) as u32, 1, 1))
        .build(vulkan);
    job.upload_buffers();
    job.build_shader();
    // job.update_ubo(0, 1, 4.0);
    job.execute();
    while job.status() == wyzoid::high::job::JobStatus::EXECUTING {
        job.wait_until_idle(1 * 1000 * 1000 * 1000);
    }
    let shader_output = job.get_output().unwrap();
    let timings = job.get_timing();
    println!("Timings:\n{}", timings);
    for i in 0..DATA_LEN {
        println!("[{}] in: {}, out: {}", i, input[i], shader_output[0][i]);
    }
}