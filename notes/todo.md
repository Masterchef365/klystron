# Todo
* Switch from free() to drop()
* Materials
* Meshes
* Each backend has a `recreate_swapchain()` function that populates the `swapchain_images` field in Core.
* Winit backend
    * `next_frame()`
    * `Swapchain`
* OpenXR backend
    * `next_frame()`
    * `Swapchain`
* Core
    * Write Command buffers
    * NOTE: MUST WRITE `vkCmdSetViewport` AND 
* logging ideas:
    * OpenXR loader path  
    * OpenXR version
    * Vulkan version
    * Number of swapchain frames
    * Log whenever a shader is loaded/unloaded and its handle
    * Log whenever a mesh is loaded/unloaded and its handle (vertex/index count)
    * OpenXR state transitions
    * Swapchain rebuilds
    * Make sure its 

## Design experiments
* Improve `AllocatedBuffer`! (see source)
* Separate features for VR and Windowed modes. You may use both if you enable both.
* Move `Mutex<Allocator>` into `VkPrelude`
* Return `Arc<VkPrelude>` from `VkPrelude::new()`. Or something. I don't like having to add Arc to everything.
* Bring Your Own Shader Uniforms (User defined data...)
