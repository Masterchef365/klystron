use crate::core::VkPrelude;
use anyhow::Result;
use erupt::{utils, vk1_0 as vk};
use std::ffi::CString;
use std::sync::Arc;
use crate::Mesh;

pub struct ParticleSet {
    mesh: Mesh,
}

pub struct ParticleSystem {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    prelude: Arc<VkPrelude>,
}

pub struct Particle {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub mass: f32,
    pub charge: f32,
}

impl ParticleSet {
    pub fn new(prelude: VkPrelude, particles: &[Particle]) -> Self {
        todo!()
    }
}
