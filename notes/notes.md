* Make an SSBO. 
* Populate the SSBO with particles
* Load up the compute shader
* Set up the compute shader pipeline
* Maybe an "Effect" abstraction?
Actually.... Maybe we have an SSBO for data the compute shader uses internally, but then the compute shader also writes to the vertex buffer. That way we can separate the stuffs. So maybe this is a ParticleSystem

* You can add a particle system, and that returns a handle. Then you can specify the particle system in the frame packet and it'll draw. Custom shader? I guess??? Frricc
Your render engine is not well set up for this...
