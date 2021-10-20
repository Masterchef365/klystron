use log::info;
use openxr::Entry;
use std::path::Path;

/// A container for several commonly-used OpenXR constants.
pub struct XrPrelude {
    pub instance: xr::Instance,
    pub session: xr::Session<xr::Vulkan>,
    pub system: xr::SystemId,
}

/// Attempt to load OpenXR dll first from OPENXR_LOADER, or the default location if no environment
/// variable is provided.
pub fn load_openxr() -> anyhow::Result<xr::Entry> {
    Ok(Entry::linked())
}
