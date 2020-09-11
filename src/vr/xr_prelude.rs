use openxr::Entry;
use std::path::Path;
use log::info;

/// A container for several commonly-used OpenXR constants.
pub struct XrPrelude {
    pub instance: xr::Instance,
    pub session: xr::Session<xr::Vulkan>,
    pub system: xr::SystemId,
}

/// Attempt to load OpenXR dll first from OPENXR_LOADER, or the default location if no environment
/// variable is provided.
pub fn load_openxr() -> anyhow::Result<xr::Entry> {
    let path = std::env::var("OPENXR_LOADER");
    use std::env::VarError;
    Ok(match path {
        Ok(path) => {
            info!("Loading OpenXR loader from {}", path);
            Entry::load_from(Path::new(&path))?
        },
        Err(VarError::NotPresent) => Entry::load()?,
        Err(e) => Err(e)?,
    })
}
