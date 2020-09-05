Steps:
* Inventory the peices of the original engine
    * Find the requirements of the swapchains for both
* Inventory the peices present in klystron
* You need to figure out how to gracefully handle OpenXR being both the camera and the 

Original engine:
* Engine core
* Frame sync
* Swapchain
* Pipeline
* Vertex
* AllocatedBuffer
* Windowed mode
    * Hardware query
    * Swapchain
    * Camera
* VR
    * Swapchain

What kind of interface we want:
* User in control of main loop
* Not _too_ much abstractino
* Not too many features. Modifiable things should be vertex and fragment shaders, meshes, time, transform matricies, and draw mode
    * Mesh: Vertices and indices
    * Material: Fragment, vertex shaders and draw mode
    * Objects: Mesh and Material references, along with transforms
    * Per-frame: Time, camera position

* Object-oriented is bad, but responsibilities are good. Maybe try putting the engine core in an Rc<T>
* KISS, just use a normal array for your frame shit.

Approach:
* Don't start with files
* Begin top down and then move bottom up

Anatomy of a frame:
* Get a frame from the swapchain. If it's out of date, try to rebuild it.
* Build the command buffer from objects
* Write camera transforms to uniforms
    * This is extremely timing sensitive in VR. It must be read and written RIGHT BEFORE QUEUE SUBMIT!
* Submit to queue
* Submit to swapchain

Implementation notes:
* Use VK\_DYNAMIC\_STATE\_VIEWPORT
    * Avoid having to recreate pipelines _ever_. Probably.
    * Materials will never be stored
    * The render pass is created once... I think?
    * Remeber to vkCmdSetScissor!!!
* You will of course have to recreate the depth images. Darn.
