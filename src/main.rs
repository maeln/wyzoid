extern crate ash;
// extern crate csv;

mod high;
mod low;
mod utils;
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
    let mut hello: Vec<f32> = Vec::with_capacity(BUFFER_CAPACITY as usize);
    for i in 0..BUFFER_CAPACITY {
        hello.push(i as f32);
    }

    let (shader_output, timings) = high::job::one_shot_job(
        &PathBuf::from("shaders/bin/double/double.cs.spirv"),
        &hello,
        ((BUFFER_CAPACITY / 4 / 64) as u32, 1, 1),
    );
    println!("[NFO] Timings:\n{}", timings);

    let new_start = Instant::now();
    let mut buf_ptr = hello.as_mut_ptr();
    for i in 0..(BUFFER_CAPACITY as usize / 4) {
        buf_ptr = doit(buf_ptr, i as f32);
    }
    println!("[NFO] CPU version: {} ms", get_fract_s(new_start.elapsed()));
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
