Questions:
* Should OpenXR be available outside the engine? I.E. you can pass it an instance like Winit?
* Well, there's going to be different draw code in each. Creating the command buffers should be about the same... 

Let's talk usecases. What do I want to use it for?
* Visualizations
    * In VR and outside of VR

Design considerations:
* Materials define how pipelines should be created
    * Pipelines must be recreated when the swapchain resizes
    * When a material is unloaded, it must also destroy the associated pipeline
* Command buffers are written every frame
* Meshes are handles for GPU allocations of vertices and indices
* Some things are in use during rendering, and only a specific instance will be writable at any given time - based on frame.
    * Command buffers
    * UBOs
    * Dynamic vertex buffers? Nah
* Number of frames never changes, but number of swapchain images may

I have abstration layers for:
* Things that are keyed to handles
* Things that are keyed to specific frames and can be updated in real time
Should hopefully be made out of reusable parts?
Interfaces... Hm...
With VR, some things are time-critical
Might need a contract of some sort...
Or you're just straight up trying to think too hard about it. Just make the core implement that stuff in its own function and badda bing badda boom you got it goin
There are some resources that are kind of contextual, primarily `device`.
Others require specifics, like command buffers

It's interesting to think about the shape of the program... Like what data is needed just when. Lots of things are just singleton resources, like command pools (usually). 
