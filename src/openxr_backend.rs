use crate::core::{Core, VkPrelude};
use crate::openxr_caddy::{load_openxr, OpenXr};
use crate::{DrawType, Engine, FramePacket, Material, Mesh, Vertex};
use anyhow::{bail, Result};
use erupt::{
    cstr, utils::allocator, vk1_0 as vk, vk1_1, DeviceLoader, EntryLoader, InstanceLoader,
};
use log::info;
use std::ffi::CString;

/// VR Capable OpenXR engine backend
pub struct OpenXrBackend {
    core: Core,
    frame_wait: xr::FrameWaiter,
    frame_stream: xr::FrameStream<xr::Vulkan>,
    stage: xr::Space,
    //swapchain: Swapchain,
}

impl OpenXrBackend {
    /// Create a new engine instance. Returns the OpenXr caddy for use with input handling.
    pub fn new(application_name: &str) -> Result<(Self, OpenXr)> {
        // Load OpenXR runtime
        let xr_entry = load_openxr()?;

        let mut enabled_extensions = xr::ExtensionSet::default();
        enabled_extensions.khr_vulkan_enable = true;
        let xr_instance = xr_entry.create_instance(
            &xr::ApplicationInfo {
                application_name,
                application_version: 0,
                engine_name: crate::ENGINE_NAME,
                engine_version: 0,
            },
            &enabled_extensions,
            &[],
        )?;
        let instance_props = xr_instance.properties()?;

        info!(
            "Loaded OpenXR runtime: {} {}",
            instance_props.runtime_name, instance_props.runtime_version
        );

        let system = xr_instance
            .system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)
            .unwrap();

        // Load Vulkan
        let vk_entry = EntryLoader::new()?;

        // Check to see if OpenXR and Vulkan are compatible
        let vk_version = unsafe { vk_entry.enumerate_instance_version(None).result()? };

        let vk_version = xr::Version::new(
            vk::version_major(vk_version) as u16,
            vk::version_minor(vk_version) as u16,
            vk::version_patch(vk_version),
        );

        info!("Loaded Vulkan version {}", vk_version);
        let reqs = xr_instance
            .graphics_requirements::<xr::Vulkan>(system)
            .unwrap();
        if reqs.min_api_version_supported > vk_version {
            bail!(
                "OpenXR runtime requires Vulkan version > {}",
                reqs.min_api_version_supported
            );
        }

        // Vulkan instance extensions required by OpenXR
        let vk_instance_exts = xr_instance
            .vulkan_instance_extensions(system)
            .unwrap()
            .split(' ')
            .map(|x| std::ffi::CString::new(x).unwrap())
            .collect::<Vec<_>>();

        let mut vk_instance_ext_ptrs = vk_instance_exts
            .iter()
            .map(|x| x.as_ptr())
            .collect::<Vec<_>>();

        let mut vk_instance_layers_ptrs = Vec::new();

        // Vulkan device extensions required by OpenXR
        let vk_device_exts = xr_instance
            .vulkan_device_extensions(system)
            .unwrap()
            .split(' ')
            .map(|x| CString::new(x).unwrap())
            .collect::<Vec<_>>();

        let mut vk_device_ext_ptrs = vk_device_exts
            .iter()
            .map(|x| x.as_ptr())
            .collect::<Vec<_>>();

        let mut vk_device_layers_ptrs = Vec::new();

        crate::extensions::extensions_and_layers(
            &mut vk_instance_layers_ptrs,
            &mut vk_instance_ext_ptrs,
            &mut vk_device_layers_ptrs,
            &mut vk_device_ext_ptrs,
        );

        // Vulkan Instance
        let application_name = CString::new(application_name)?;
        let engine_name = CString::new(crate::ENGINE_NAME)?;
        let app_info = vk::ApplicationInfoBuilder::new()
            .application_name(&application_name)
            .application_version(vk::make_version(1, 0, 0))
            .engine_name(&engine_name)
            .engine_version(crate::engine_version())
            .api_version(vk::make_version(1, 1, 0));

        let create_info = vk::InstanceCreateInfoBuilder::new()
            .application_info(&app_info)
            .enabled_layer_names(&vk_instance_layers_ptrs)
            .enabled_extension_names(&vk_instance_ext_ptrs);

        let vk_instance = InstanceLoader::new(&vk_entry, &create_info, None)?;

        // Obtain physical vk_device, queue_family_index, and vk_device from OpenXR
        let vk_physical_device = vk::PhysicalDevice(
            xr_instance
                .vulkan_graphics_device(system, vk_instance.handle.0 as _)
                .unwrap() as _,
        );

        let queue_family_index = unsafe {
            vk_instance
                .get_physical_device_queue_family_properties(vk_physical_device, None)
                .into_iter()
                .enumerate()
                .filter_map(|(queue_family_index, info)| {
                    if info.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                        Some(queue_family_index as u32)
                    } else {
                        None
                    }
                })
                .next()
                .expect("Vulkan vk_device has no graphics queue")
        };

        let mut create_info = vk::DeviceCreateInfoBuilder::new()
            .queue_create_infos(&[vk::DeviceQueueCreateInfoBuilder::new()
                .queue_family_index(queue_family_index)
                .queue_priorities(&[1.0])])
            .enabled_layer_names(&vk_device_layers_ptrs)
            .enabled_extension_names(&vk_device_ext_ptrs)
            .build();

        let mut phys_device_features = erupt::vk1_2::PhysicalDeviceVulkan11Features {
            multiview: vk::TRUE,
            ..Default::default()
        };

        create_info.p_next = &mut phys_device_features as *mut _ as _;

        let vk_device = DeviceLoader::new(&vk_instance, vk_physical_device, &create_info, None)?;
        let queue = unsafe { vk_device.get_device_queue(queue_family_index, 0, None) };

        let _ = VkPrelude {
            queue,
            queue_family_index,
            device: vk_device,
            physical_device: vk_physical_device,
            instance: vk_instance,
            entry: vk_entry,
        };

        /*
        let (session, frame_wait, frame_stream) = unsafe {
            xr_instance.create_session::<xr::Vulkan>(
                system,
                &xr::vulkan::SessionCreateInfo {
                    instance: vk_instance.handle.0 as _,
                    physical_device: vk_physical_device.0 as _,
                    device: vk_device.handle.0 as _,
                    queue_family_index,
                    queue_index: 0,
                },
            )
        }?;

        let stage = session
            .create_reference_space(xr::ReferenceSpaceType::STAGE, xr::Posef::IDENTITY)
            .unwrap();
        */

        todo!()
    }

    /// Render a frame of video.
    /// Returns false when the loop should break
    pub fn next_frame(&mut self, openxr: &OpenXr, packet: &FramePacket) -> Result<bool> {
        todo!()
    }
}

impl Engine for OpenXrBackend {
    fn add_material(
        &mut self,
        vertex: &[u8],
        fragment: &[u8],
        draw_type: DrawType,
    ) -> Result<Material> {
        todo!()
    }
    fn add_mesh(&mut self, vertices: &[Vertex], indices: &[u16]) -> Result<Mesh> {
        todo!()
    }
    fn remove_material(&mut self, material: Material) {
        todo!()
    }
    fn remove_mesh(&mut self, mesh: Mesh) {
        todo!()
    }
}
