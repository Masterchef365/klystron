# Todo
* Caddy
    * Creation, destruction
* Core
    * Setup
    * Destruction
    * Allocator/AllocatedBuffer
        * Maybe just use different types for GPU-only index/vertex buffers and UBOs. K.I.S.S.
    * Materials
    * Camera UBOs
    * Write Command buffers
* Winit backend
* OpenXR backend
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
* Separate features for VR and Windowed modes. You may use both if you enable both.
