use anyhow::Result;
use erupt::{
    utils::allocator::{self, Allocator},
    vk1_0 as vk, DeviceLoader,
};
use std::marker::PhantomData;

// TODO:
// * Map on construction and only map once!
// * GPU-only memory

/// A buffer and its associated allocation on device.
pub struct AllocatedBuffer<T> {
    pub buffer: vk::Buffer,
    pub allocation: Option<allocator::Allocation<vk::Buffer>>,
    _phantom: PhantomData<T>,
    freed: bool,
}

impl<T: Sized + bytemuck::Pod> AllocatedBuffer<T> {
    /// Create a new buffer able to contain `count` instance of `T`
    pub fn new(
        count: usize,
        create_info: vk::BufferCreateInfoBuilder,
        allocator: &mut Allocator,
        device: &DeviceLoader,
    ) -> Result<Self> {
        anyhow::ensure!(count > 0, "Must allocate at least one object");
        let size = std::mem::size_of::<T>() * count;
        let create_info = create_info.size(size as u64);
        let buffer = unsafe { device.create_buffer(&create_info, None, None) }.result()?;
        let allocation = allocator
            .allocate(&device, buffer, allocator::MemoryTypeFinder::dynamic())
            .result()?;
        Ok(Self {
            buffer,
            allocation: Some(allocation),
            freed: false,
            _phantom: PhantomData::default(),
        })
    }

    pub fn map(&self, device: &DeviceLoader, data: &[T]) -> Result<()> {
        let mut map = self
            .allocation
            .as_ref()
            .expect("Use-after-free")
            .map(device, ..)
            .result()?;
        map.import(bytemuck::cast_slice(data));
        map.unmap(&device).result()?;
        Ok(())
    }

    pub fn free(&mut self, device: &DeviceLoader, allocator: &mut Allocator) -> Result<()> {
        unsafe {
            device.device_wait_idle().result()?;
        }
        allocator.free(
            &device,
            self.allocation.take().expect("Already deallocated"),
        );
        self.freed = true;
        Ok(())
    }
}

impl<T> Drop for AllocatedBuffer<T> {
    fn drop(&mut self) {
        if !self.freed {
            panic!(
                "AllocatedBuffer<{}> was dropped before it was freed!",
                std::any::type_name::<T>()
            );
        }
    }
}
