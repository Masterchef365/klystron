# Todo
* Conundrum with the `image_available` semaphore not being available. Likely has to be a polyfill.
* Arcball camera jig for windowed mode. Scrape code from lucidyne stuff.
* Use dropbomb instead of a boolean for some stuff like allocatedbuffer I think
* Add `Engine` methods to Core. Consider just exposing it to the user.
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
    * NOTE: MUST WRITE `vkCmdSetViewport` AND `vkCmdSetScissor`
* logging ideas:
    * OpenXR loader path  
    * OpenXR version
    * Vulkan version
    * Number of swapchain frames
    * Log whenever a shader is loaded/unloaded and its handle
    * Log whenever a mesh is loaded/unloaded and its handle (vertex/index count)
    * OpenXR state transitions
    * Swapchain rebuilds

## Design experiments
* Improve `AllocatedBuffer`! (see source)
* Separate features for VR and Windowed modes. You may use both if you enable both.
* Move `Mutex<Allocator>` into `VkPrelude`
* Return `Arc<VkPrelude>` from `VkPrelude::new()`. Or something. I don't like having to add Arc to everything.
* Bring Your Own Shader Uniforms (User defined data...)
* Object for each backend, just pass the core in an argument to `next_frame()`
