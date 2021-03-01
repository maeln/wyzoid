use ash::extensions::khr::Display;
use log::error;
use rand::Rng;
use std::ffi::CString;
use std::fmt::Display as FmtDisplay;
use std::ops::{Add, Div, Mul, Sub};
use std::path::PathBuf;
use std::time::Duration;

pub fn to_vec32(vecin: Vec<u8>) -> Vec<u32> {
    unsafe { vecin.align_to::<u32>().1.to_vec() }
}

pub fn load_file(file: &PathBuf) -> Option<Vec<u8>> {
    let contents = std::fs::read(file);
    match contents {
        Ok(file_str) => Some(file_str),
        Err(err) => {
            error!("Impossible to read file {} : {}", file.display(), err);

            None
        }
    }
}

pub fn tick(val: bool) -> String {
    if val {
        return "✅".to_string();
    }
    "❌".to_string()
}

pub fn cstr2string(mut cstr: Vec<i8>) -> String {
    let string = unsafe { CString::from_raw(cstr.as_mut_ptr()) };
    std::mem::forget(cstr);
    String::from(string.to_string_lossy())
}

pub fn get_fract_s(date: Duration) -> String {
    let millis = date.subsec_millis() as u64;
    let sec = date.as_secs();
    let tot = sec * 1000 + millis;
    format!("{}", tot)
}

// Bad f32 comparison with a epsilon
pub fn f32_cmp(a: f32, b: f32, epsilon: f32) -> bool {
    (a + epsilon) > b && b > (a - epsilon)
}

pub fn rand_vec<T>(len: usize, low: T, high: T) -> Vec<T>
where
    T: rand::distributions::uniform::SampleUniform + Copy + PartialOrd,
{
    let mut rng = rand::thread_rng();
    let mut output: Vec<T> = Vec::with_capacity(len);

    for _ in 0..len {
        output.push(rng.gen_range(low..high))
    }

    output
}

pub fn min_max<T: PartialOrd + Copy>(data: &[T]) -> Option<(T, T)> {
    if data.is_empty() {
        return None;
    } else if data.len() == 1 {
        return Some((data[0], data[0]));
    }

    let mut min_local: T = data[0];
    let mut max_local: T = data[0];
    for i in 1..data.len() {
        if min_local > data[i] {
            min_local = data[i];
        }
        if max_local < data[i] {
            max_local = data[i];
        }
    }

    Some((min_local, max_local))
}

pub fn remap<T>(x: T, origin_min: T, origin_max: T, map_min: T, map_max: T) -> T
where
    T: Copy + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>,
{
    map_min + (x - origin_min) * (map_max - map_min) / (origin_max - origin_min)
}

pub fn to_ppm(data: &Vec<f32>, width: usize, height: usize) -> Option<String> {
    if width * height * 3 != data.len() {
        return None;
    }

    let (min, max) = min_max(data).unwrap();
    let mut ppm = String::new();
    ppm.push_str("P3\n");
    ppm.push_str(&format!("{} {}\n", width, height));
    ppm.push_str("255\n");

    for i in 0..data.len() {
        ppm.push_str(&format!("{} ", remap(data[i], min, max, 0.0, 255.0) as u8));
        if i % width == 0 {
            ppm.push_str("\n");
        }
    }

    Some(ppm)
}

pub fn to_csv<T>(label: &str, values: &Vec<T>) -> String
where
    T: FmtDisplay,
{
    let mut csv = String::new();
    csv.push_str(label);
    csv.push(';');
    for val in values {
        csv.push_str(&format!("{};", val));
    }
    csv.push('\n');
    return csv;
}
