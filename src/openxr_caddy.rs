use anyhow::Result;

pub struct OpenXr {
    pub xr_instance: xr::Instance,
    pub session: xr::Session<xr::Vulkan>,
    pub system: xr::SystemId,
}

impl OpenXr {
    pub fn new(name: &str) -> Result<Self> {
        todo!()
    }
}
