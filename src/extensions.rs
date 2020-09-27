use erupt::cstr;

pub fn extensions_and_layers(
    instance_layers: &mut Vec<*const i8>,
    instance_extensions: &mut Vec<*const i8>,
    device_layers: &mut Vec<*const i8>,
    _device_extensions: &mut Vec<*const i8>,
) {
    // Vulkan layers and extensions
    const LAYER_KHRONOS_VALIDATION: *const i8 = cstr!("VK_LAYER_KHRONOS_validation");

    //if cfg!(debug_assertions) {
    instance_extensions.push(erupt::extensions::ext_debug_utils::EXT_DEBUG_UTILS_EXTENSION_NAME);
    instance_layers.push(LAYER_KHRONOS_VALIDATION);
    device_layers.push(LAYER_KHRONOS_VALIDATION);
    //}
}
