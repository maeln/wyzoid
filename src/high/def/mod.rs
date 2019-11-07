use crate::high::states;
use crate::low::vkmem;
use crate::low::vkshader;
use crate::low::vkstate;
pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
pub use ash::vk;
use std::convert::Into;
use std::ffi::CString;
use std::os::raw::c_void;
use std::path::PathBuf;
use std::rc::Rc;

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

/// The two buffer type supported: SSBOs and Uniforms.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BufferType {
    SSBO,
    UNIFORM,
}

impl Into<vk::BufferUsageFlags> for BufferType {
    fn into(self) -> vk::BufferUsageFlags {
        match self {
            SSBO => vk::BufferUsageFlags::STORAGE_BUFFER,
            UNIFORM => vk::BufferUsageFlags::UNIFORM_BUFFER,
        }
    }
}

trait Bufferizable: Sized {}

/// A high-level structure that will represent a buffer that can be uploaded
/// to the GPU memory.
/// NOTE: A uniform is just a buffer where T implement `Serializable`
struct Buffer {
    /// The data to upload. Note that the `Buffer` own its `Vec`.
    data: *const c_void,
    size: usize,
    buffer_type: BufferType,
}

impl Buffer {
    pub fn new(data: *const c_void, size: usize, buffer_type: BufferType) -> Buffer {
        Buffer {
            data,
            size,
            buffer_type,
        }
    }
}

/// A shader execution definition with its path and dispatch.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Shader {
    path: PathBuf,
    dispatch: Dispatch,
}

impl Shader {
    pub fn new(path: PathBuf, dispatch: (u32, u32, u32)) -> Shader {
        Shader {
            path,
            dispatch: Dispatch::new(dispatch.0, dispatch.1, dispatch.2),
        }
    }
}

struct Link {
    shader_id: usize,
    buffer_id: usize,
    bind: BindPoint,
}

impl Link {
    pub fn new(shader_id: usize, buffer_id: usize, bind: BindPoint) -> Link {
        Link {
            shader_id,
            buffer_id,
            bind,
        }
    }
}

const ENTRY_POINT: CString = CString::new("main").unwrap();

struct JobDefinition {
    shaders: Vec<Shader>,
    buffers: Vec<Buffer>,
    links: Vec<Link>,
    vk_buffers: Vec<vkmem::VkBuffer>,
    vk_memory: Option<vkmem::VkMem>,
    vk_shaders: Vec<vkshader::VkShader>,
}

impl JobDefinition {
    pub fn new() -> JobDefinition {
        JobDefinition {
            shaders: Vec::new(),
            buffers: Vec::new(),
            links: Vec::new(),
            vk_buffers: Vec::new(),
            vk_memory: None,
            vk_shaders: Vec::new(),
        }
    }

    pub fn add_shader(&mut self, shader: Shader) -> usize {
        self.shaders.push(shader);
        self.shaders.len() - 1
    }

    pub fn add_buffer(&mut self, buffer: Buffer) -> usize {
        self.buffers.push(buffer);
        self.buffers.len() - 1
    }

    pub fn link(&mut self, shader_id: usize, buffer_id: usize, bind: BindPoint) -> usize {
        self.links.push(Link::new(shader_id, buffer_id, bind));
        self.links.len() - 1
    }

    fn size(&self) -> usize {
        self.buffers.iter().fold(0, |acc, x| acc + x.size)
    }

    pub fn compile_buffers(&mut self, state: Rc<vkstate::VulkanState>) {
        self.vk_buffers = self
            .buffers
            .iter()
            .map(|b| vkmem::VkBuffer::new(state, b.size as u64, b.buffer_type.into()))
            .collect();
    }

    pub fn find_mem(&mut self, state: Rc<vkstate::VulkanState>) {
        let (mem_size, offsets) = vkmem::compute_non_overlapping_buffer_alignment(
            &self.vk_buffers.iter().map(|b| b).collect(),
        );
        self.vk_memory = Some(
            vkmem::VkMem::find_mem(state, mem_size)
                .expect("[ERR] Could not find a memory type fitting our need."),
        );
    }

    pub fn compile_shaders(&mut self, state: Rc<vkstate::VulkanState>) {
        self.vk_shaders = self
            .shaders
            .iter()
            .map(|s| vkshader::VkShader::new(state, &s.path, ENTRY_POINT))
            .collect();
    }
}
