# Todo
* Switch from free() to drop()
* Materials
* Meshes
* Each backend has a recreate\_swapchain() function that populates the swapchain\_images field in Core.
* Winit backend
    * next\_frame()
    * Swapchain
* OpenXR backend
    * next\_frame()
    * Swapchain
* Core
    * Write Command buffers
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
* Improve AllocatedBuffer! (see source)
* Separate features for VR and Windowed modes. You may use both if you enable both.
* Experiment by moving `Mutex<Allocator>` into `VkPrelude`, with
