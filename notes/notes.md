God DAMNIT
Okay so actually, we gotta use two entry points and a pipeline barrier
Use ONLY ONE descriptor set. It's the same...? Well it's close enough sorta
uhh

Two pipelines... FUCK

```
Barrier (Nothing -> Compute)
    * SSBO (AccessFlags: Empty -> SHADER_WRITE)

for particle_system in particle_systems:
    Bind forces pipeline
    Dispatch forces

Barrier (Compute -> Compute)
    * SSBO (AccessFlags: SHADER_WRITE -> SHADER_WRITE)
    * Vertex buffer (AccessFlags: Empty -> SHADER_WRITE)

for particle_system in particle_systems:
    Bind motion pipeline
    Dispatch motion

Barrier (Compute -> Render pass...)
    * SSBO (AccessFlags: SHADER_WRITE -> SHADER_READ)
    * Vertex buffer (AccessFlags: SHADER_WRITE -> SHADER_READ)

// Draw, having bound the SSBO to a descriptor 
```


Need to actually write command buffers.. But before then we should make that loading this all works

Big problem: `FRAMES_IN_FLIGHT` must equal one. This is because our current synchronization model will begin a new frame before the last one finished.
That might kill VR until you can work out a better solution.

* Why do we make a new pipeline layout for every material?
    * We really only need one per draw type


* Make an SSBO. 
* Populate the SSBO with particles
* Load up the compute shader
* Set up the compute shader pipeline
* Maybe an "Effect" abstraction?
Actually.... Maybe we have an SSBO for data the compute shader uses internally, but then the compute shader also writes to the vertex buffer. That way we can separate the stuffs. So maybe this is a ParticleSystem

* You can add a particle system, and that returns a handle. Then you can specify the particle system in the frame packet and it'll draw. Custom shader? I guess??? Frricc
Your render engine is not well set up for this...

Y'know, you might be able to increase the number of visible "points" somehow by instead abusing fragment shaders to create a parallax effect with a texture of points. This would be much harder in VR I'd imagine... Well actually... If you had a seed for the random function, and used only camera-relative positional data it might work just fine. Hm.

OOH - Once you have the particle system working, you should try setting it to triangle mode lol

Remove the need for an index buffer perhaps... Just use draw instead of drawIndexed
