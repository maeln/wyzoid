# 0.1.0

    - Initial release

# 0.1.1

    - Add the possibility to use "read-only" buffer. Those buffers won't be initialized with data, skipping the memory mapping stage. Useful when a compute shader is used to fill the buffer.
    - Add a "dataviz" exemple using this feature.

# 0.1.2

    - Use `log` to display runtime info instead of `println!`.

# 0.1.3 - WIP

    - Executing a job is now asynchronous using fences.
    - The vulkan state now has to be initialized outside of a job.
