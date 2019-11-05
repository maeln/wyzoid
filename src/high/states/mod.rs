use crate::low::{vkdescriptor, vkmem, vkpipeline, vkshader};
pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use std::cell::RefCell;
use std::rc::Rc;

/// Every object related to the shader execution
struct ShaderState {
    /// Reference to the shader module
    shader: Rc<RefCell<vkshader::VkShader>>,
    /// Descriptors of the shader
    descriptor: vkdescriptor::VkDescriptor,
    /// Write descriptors of the shader
    write_descriptor: vkdescriptor::VkWriteDescriptor,
    /// Pipeline of the shader
    pipeline: vkpipeline::VkComputePipeline,
}

/// A builder for `ShaderState`
struct ShaderStateBuilder {
    shader: Option<Rc<RefCell<vkshader::VkShader>>>,
    descriptor: Option<vkdescriptor::VkDescriptor>,
    write_descriptor: Option<vkdescriptor::VkWriteDescriptor>,
    pipeline: Option<vkpipeline::VkComputePipeline>,
}

impl ShaderStateBuilder {
    pub fn new() -> ShaderStateBuilder {
        ShaderStateBuilder {
            shader: None,
            descriptor: None,
            write_descriptor: None,
            pipeline: None,
        }
    }

    pub fn shader(mut self, shader: Rc<RefCell<vkshader::VkShader>>) -> ShaderStateBuilder {
        self.shader = Some(shader.clone());
        self
    }

    pub fn descriptor(mut self, descriptor: vkdescriptor::VkDescriptor) -> ShaderStateBuilder {
        self.descriptor = Some(descriptor);
        self
    }

    pub fn write_descriptor(
        mut self,
        write_descriptor: vkdescriptor::VkWriteDescriptor,
    ) -> ShaderStateBuilder {
        self.write_descriptor = Some(write_descriptor);
        self
    }

    pub fn pipeline(mut self, pipeline: vkpipeline::VkComputePipeline) -> ShaderStateBuilder {
        self.pipeline = Some(pipeline);
        self
    }

    pub fn build(self) -> Option<ShaderState> {
        if self.shader.is_none() || self.descriptor.is_none() || self.write_descriptor.is_none() {
            return None;
        }

        Some(ShaderState {
            shader: self.shader.unwrap(),
            descriptor: self.descriptor.unwrap(),
            write_descriptor: self.write_descriptor.unwrap(),
            pipeline: self.pipeline.unwrap(),
        })
    }
}

/// The vulkan memory state
struct MemoryState {
    memory: vkmem::VkMem,
    buffers: Vec<vkmem::VkBuffer>,
}

/// A builder for `MemoryState`
struct MemoryStateBuilder {
    memory: Option<vkmem::VkMem>,
    buffers: Vec<vkmem::VkBuffer>,
}

impl MemoryStateBuilder {
    pub fn new() -> MemoryStateBuilder {
        MemoryStateBuilder {
            memory: None,
            buffers: Vec::new(),
        }
    }

    pub fn memory(mut self, mem: vkmem::VkMem) -> MemoryStateBuilder {
        self.memory = Some(mem);
        self
    }

    pub fn add_buffer(mut self, buffer: vkmem::VkBuffer) -> MemoryStateBuilder {
        self.buffers.push(buffer);
        self
    }

    pub fn build(self) -> Option<MemoryState> {
        if self.memory.is_none() {
            return None;
        }

        Some(MemoryState {
            memory: self.memory.unwrap(),
            buffers: self.buffers,
        })
    }
}

struct ExecutionState {
    memory_state: MemoryState,
    shaders_states: Vec<ShaderState>,
}

struct ExecutionStateBuilder {
    memory_state: Option<MemoryState>,
    shaders_states: Vec<ShaderState>,
}

impl ExecutionStateBuilder {
    pub fn new() -> ExecutionStateBuilder {
        ExecutionStateBuilder {
            memory_state: None,
            shaders_states: Vec::new(),
        }
    }

    pub fn memory_state(mut self, state: MemoryState) -> ExecutionStateBuilder {
        self.memory_state = Some(state);
        self
    }

    pub fn add_shaders_states(mut self, state: ShaderState) -> ExecutionStateBuilder {
        self.shaders_states.push(state);
        self
    }

    pub fn build(self) -> ExecutionState {
        ExecutionState {
            memory_state: self.memory_state.unwrap(),
            shaders_states: self.shaders_states,
        }
    }
}
