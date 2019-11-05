use crate::low::vkmem;
use std::path::PathBuf;

/// Represent a shader binding point
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct BindPoint {
    /// Set to bind to
    set: u32,
    /// Binding point within the set
    bind: u32,
}

impl BindPoint {
    pub fn new(set: u32, bind: u32) -> BindPoint {
        BindPoint { set, bind }
    }
}

/// Represent a UBO
pub struct Uniform<'a> {
    /// Where to bind the UBO
    bind: BindPoint,
    /// The UBO data. Has to be a struct that implement vkmem::Serializable
    data: &'a dyn vkmem::Serializable,
}

/// A high-level structure that will represent a buffer that can be uploaded
/// to the GPU memory.
pub struct Buffer<T> {
    /// The bind point of the buffer on the GPU
    bind: BindPoint,
    /// The data to upload. Note that the `Buffer` own its `Vec`.
    data: Vec<T>,
}

impl<T> Buffer<T> {
    /// Create a new buffer.
    /// # Arguments
    /// * `bind` - The bindpoint of the buffer on the GPU
    /// * `data` - The data of the buffer.
    /// `Buffer` will copy the `Vec` meaning the original `Vec` can be safely discarded after.
    pub fn new(bind: BindPoint, data: &Vec<T>) -> Buffer<T> {
        Buffer {
            bind,
            data: *data.clone(),
        }
    }

    /// The size (in octet) of the buffer
    pub fn size(&self) -> usize {
        std::mem::size_of::<T>() * self.data.len()
    }
}

/// A dispatch is a 3D vector representing the workgroup executions for the shader.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Dispatch {
    x: u32,
    y: u32,
    z: u32,
}

impl Dispatch {
    pub fn new(x: u32, y: u32, z: u32) -> Dispatch {
        Dispatch { x, y, z }
    }
}

/// A shader execution definition with its path and dispatch.
struct ShaderExecution<'a> {
    shader: PathBuf,
    dispatch: Dispatch,
    uniforms: Vec<Uniform<'a>>,
}

struct JobDefinition<'a, T> {
    buffers: Vec<Buffer<T>>,
    shaders_executions: ShaderExecution<'a>,
}
