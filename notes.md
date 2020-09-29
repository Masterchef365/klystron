Create postprocessing image
Fragment writes to postprocessing image
(Barrier) Swapchain `PRESENT_KHR` -> `GENERAL`
Compute shader reads from postproc and outputs to swapchain image
(Barrier) Swapchain `GENERAL` -> `PRESENT_KHR`

Simplify framebuffer; there is only one not one per frame. The compute shader stuff handles getting the right image. So there should be... N ?

Test by inserting one barrier that.. That isn't a copy
Fuc
Fiiiine I'll do compute shader first

```rust
let postprocessing = engine.add_compute_shader(fs::read("...")?);
FramePacket {
    objects,
    postprocessing,
}
```

```rust
struct FramePacket {
    objects: Vec<Object>,
    postprocessing: ComputeShader,
}
```

Descriptor set for compute/postprocessing pipeline
Needs to have animation in there
