use crate::frame_sync::Frame;
use anyhow::Result;
use erupt::{vk1_0 as vk, DeviceLoader};
use gpu_alloc_erupt::EruptMemoryDevice;
use vk_core::SharedCore;

pub struct SwapchainImages {
    pub extent: vk::Extent2D,
    pub depth_image: vk::Image,
    pub depth_image_mem: Option<gpu_alloc::MemoryBlock<vk::DeviceMemory>>,
    pub depth_image_view: vk::ImageView,
    images: Vec<SwapChainImage>,
    prelude: SharedCore,
}

#[derive(Copy, Clone)]
pub struct SwapChainImage {
    pub framebuffer: vk::Framebuffer,
    pub image_view: vk::ImageView,
    /// Whether or not the frame which this swapchain image is dependent on is in flight or not
    pub extent: vk::Extent2D,
    pub in_flight: vk::Fence,
}

impl SwapchainImages {
    /// Returns None if the swapchain is out of date
    pub fn next_image(&mut self, image_index: u32, frame: &Frame) -> Result<SwapChainImage> {
        let image = &mut self.images[image_index as usize];

        // Wait until the frame associated with this swapchain image is finisehd rendering, if any
        // May be null if no frames have flowed just yet
        if !image.in_flight.is_null() {
            unsafe {
                self.prelude
                    .device
                    .wait_for_fences(&[image.in_flight], true, u64::MAX)
            }
            .result()?;
        }

        // Associate this swapchain image with the given frame. When the frame is finished, this
        // swapchain image will know (see above) when this image is rendered.
        image.in_flight = frame.in_flight_fence;

        Ok(*image)
    }

    pub fn new(
        prelude: SharedCore,
        extent: vk::Extent2D,
        render_pass: vk::RenderPass,
        swapchain_images: Vec<vk::Image>,
        vr: bool,
    ) -> Result<Self> {
        let layers = if vr { 2 } else { 1 };

        // Create depth image
        let create_info = vk::ImageCreateInfoBuilder::new()
            .image_type(vk::ImageType::_2D)
            .extent(
                vk::Extent3DBuilder::new()
                    .width(extent.width)
                    .height(extent.height)
                    .depth(1)
                    .build(),
            )
            .mip_levels(1)
            .array_layers(layers)
            .format(crate::core::DEPTH_FORMAT)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .samples(vk::SampleCountFlagBits::_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let depth_image =
            unsafe { prelude.device.create_image(&create_info, None, None) }.result()?;

        let requirements = unsafe {
            prelude
                .device
                .get_image_memory_requirements(depth_image, None)
        };

        use gpu_alloc::UsageFlags as UF;
        let request = gpu_alloc::Request {
            size: requirements.size,
            align_mask: requirements.alignment,
            usage: UF::FAST_DEVICE_ACCESS,
            memory_types: requirements.memory_type_bits,
        };

        let depth_image_mem = unsafe {
            prelude
                .allocator()?
                .alloc(EruptMemoryDevice::wrap(&prelude.device), request)?
        };

        unsafe {
            prelude
                .device
                .bind_image_memory(
                    depth_image,
                    *depth_image_mem.memory(),
                    depth_image_mem.offset(),
                )
                .result()?;
        }

        let create_info = vk::ImageViewCreateInfoBuilder::new()
            .image(depth_image)
            .view_type(vk::ImageViewType::_2D)
            .format(crate::core::DEPTH_FORMAT)
            .subresource_range(
                vk::ImageSubresourceRangeBuilder::new()
                    .aspect_mask(vk::ImageAspectFlags::DEPTH)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(layers)
                    .build(),
            );
        let depth_image_view =
            unsafe { prelude.device.create_image_view(&create_info, None, None) }.result()?;

        // Build swapchain image views and buffers
        let images = swapchain_images
            .iter()
            .map(|&image| {
                SwapChainImage::new(
                    &prelude.device,
                    render_pass,
                    image,
                    extent,
                    depth_image_view,
                    vr,
                )
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            extent,
            images,
            depth_image,
            depth_image_mem: Some(depth_image_mem),
            depth_image_view,
            prelude,
        })
    }
}

impl Drop for SwapchainImages {
    fn drop(&mut self) {
        unsafe {
            self.prelude.device.device_wait_idle().result().unwrap();
            for image in self.images.drain(..) {
                self.prelude
                    .device
                    .destroy_framebuffer(Some(image.framebuffer), None);
                self.prelude
                    .device
                    .destroy_image_view(Some(image.image_view), None);
            }
            self.prelude
                .device
                .destroy_image_view(Some(self.depth_image_view), None);
            self.prelude
                .device
                .destroy_image(Some(self.depth_image), None);
        }

        unsafe {
            self.prelude.allocator().as_mut().unwrap().dealloc(
                EruptMemoryDevice::wrap(&self.prelude.device),
                self.depth_image_mem.take().unwrap(),
            );
        }
    }
}

impl SwapChainImage {
    pub fn new(
        device: &DeviceLoader,
        render_pass: vk::RenderPass,
        swapchain_image: vk::Image,
        extent: vk::Extent2D,
        depth_image_view: vk::ImageView,
        vr: bool,
    ) -> Result<Self> {
        let in_flight = vk::Fence::null();

        let create_info = vk::ImageViewCreateInfoBuilder::new()
            .image(swapchain_image)
            .view_type(vk::ImageViewType::_2D)
            .format(crate::core::COLOR_FORMAT)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(
                vk::ImageSubresourceRangeBuilder::new()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(if vr { 2 } else { 1 })
                    .build(),
            );

        let image_view = unsafe { device.create_image_view(&create_info, None, None) }.result()?;

        let attachments = [image_view, depth_image_view];
        let create_info = vk::FramebufferCreateInfoBuilder::new()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(extent.width)
            .height(extent.height)
            .layers(1);

        let framebuffer =
            unsafe { device.create_framebuffer(&create_info, None, None) }.result()?;

        Ok(Self {
            framebuffer,
            image_view,
            in_flight,
            extent,
        })
    }
}
