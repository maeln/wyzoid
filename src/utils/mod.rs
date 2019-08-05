use std::ffi::CString;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub fn to_vec32(vecin: Vec<u8>) -> Vec<u32> {
    unsafe { vecin.align_to::<u32>().1.to_vec() }
}

pub fn load_file(file: &PathBuf) -> Option<Vec<u8>> {
    let contents = std::fs::read(file);
    match contents {
        Ok(file_str) => Some(file_str),
        Err(err) => {
            eprintln!("[ERR] Impossible to read file {} : {}", file.display(), err);

            None
        }
    }
}

pub fn print_tick(val: bool) {
    if val {
        println!("✅");
    } else {
        println!("❌");
    }
}

pub fn cstr2string(mut cstr: Vec<i8>) -> String {
    let string = unsafe { CString::from_raw(cstr.as_mut_ptr()) };
    std::mem::forget(cstr);
    String::from(string.to_string_lossy())
}

pub fn get_fract_s(date: Instant) -> String {
    let duration: Duration = date.elapsed();
    let millis = duration.subsec_millis() as u64;
    let sec = duration.as_secs();
    let tot = sec * 1000 + millis;
    format!("{}", tot)
}

// Bad f32 comparison with a epsilon
pub fn f32_cmp(a: f32, b: f32, epsilon: f32) -> bool {
    (a + epsilon) > b && b > (a - epsilon)
}
