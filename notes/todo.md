# Todo
* Winit backend
    * Swapchain
* OpenXR backend
    * Swapchain
* Core
    * Write Command buffers
* Use the `log` crate; here are some logging ideas:
    * OpenXR loader path  
    * OpenXR version
    * Vulkan version
    * Number of swapchain frames
    * Log whenever a shader is loaded/unloaded and its handle
    * Log whenever a mesh is loaded/unloaded and its handle (vertex/index count)
    * OpenXR state transitions
    * Swapchain rebuilds
    * Make sure its 
* Improve AllocatedBuffer!
* Separate features for VR and Windowed modes. You may use both if you enable both.
