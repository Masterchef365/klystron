# Klystron
A little rendering engine built in Vulkan and Rust, capable of both VR and Windowed rendering. Hopefully hack-friendly. Aimed at rendering simple, unlit scenes with animated components.

## State
Extremely WIP. Does indeed draw triangles and lines and stuff!

## Running on Windows
1. Clone and compile the OpenXR Loader from https://github.com/KhronosGroup/OpenXR-SDK
    * Use `cmake -G "Visual Studio 16 2019"  -DDYNAMIC_LOADER=ON ..`
2. Set the `OPENXR_LOADER` environment variable to the location of the OpenXR loader DLL

## Portal
Command buffer:
0. Clear the stencil buffer explicitly?
1. Render the regular scene
    * Disable stencil test
2. Render Portal A mask
    * Disable stencil test
    * Set reference to 1
    * PassOp: REPLACE
    * FailOp: KEEP
    * DepthFailOp: KEEP
3. Render Portal B mask
    * Disable stencil test
    * Set reference to 2
    * PassOp: REPLACE
    * FailOp: KEEP
    * DepthFailOp: KEEP
4. Somehow fuckin clear ONLY the depth buffer??
5. Render Portal A view
    * Enable stencil test
    * Set reference to 1
    * CompareOp: EQUAL
6. Render Portal B view
    * Enable stencil test
    * Set reference to 2
    * CompareOp: EQUAL

[`VK_EXT_extended_dynamic_state`](https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VK_EXT_extended_dynamic_state.html):
vkCmdSetStencilTestEnableEXT()
vkCmdSetStencilOpEXT()
vkCmdSetStencilReference()

Scene renderpass ops:
`.load_op(vk::AttachmentLoadOp::CLEAR)`
`.store_op(vk::AttachmentStoreOp::STORE)`
`.stencil_load_op(vk::AttachmentLoadOp::LOAD)`
`.stencil_store_op(vk::AttachmentStoreOp::STORE)`

Portal mask renderpass:
`.load_op(vk::AttachmentLoadOp::LOAD)`
`.store_op(vk::AttachmentStoreOp::STORE)`
`.stencil_load_op(vk::AttachmentLoadOp::LOAD)`
`.stencil_store_op(vk::AttachmentStoreOp::STORE)`

Renderpasses:
Scene
Portal
Portal
Scene
Scene

Next steps:
* First person camera
* Finish portal motion calc
    * Debounce might be necessary
    * Apply this to the center of eyes of the VR, and to first person camera
* Enable the `VK_EXT_extended_dynamic_state` feature
* Add special render pass for portals (using the create render pass function with the portal set to true)

# Idea
Set different time in the ubo for each of the portals!
That makes it time travel on the other side...

Make it so that the portal motion for each eye in VR is independent - if you walked forward so that the edge of the portal seperated your eyes they would _actually seperate_
