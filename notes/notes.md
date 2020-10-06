Big problem: `FRAMES_IN_FLIGHT` must equal one. This is because our current synchronization model will begin a new frame before the last one finished.
That might kill VR until you can work out a better solution.


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
