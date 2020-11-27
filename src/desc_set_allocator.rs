use crate::core::VkPrelude;
use std::sync::Arc;
use anyhow::Result;
use erupt::vk1_0 as vk;

const ALLOCATION_SIZE: u32 = 15;

pub struct DescriptorSetAllocator {
    template: Vec<vk::DescriptorPoolSizeBuilder<'static>>,
    descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
    pools: Vec<vk::DescriptorPool>,
    current_sets: Vec<vk::DescriptorSet>,
    prelude: Arc<VkPrelude>,
}

impl DescriptorSetAllocator {
    pub fn new(
        mut template: Vec<vk::DescriptorPoolSizeBuilder<'static>>,
        descriptor_set_layout: vk::DescriptorSetLayout,
        prelude: Arc<VkPrelude>,
    ) -> Self {
        for dpsb in &mut template {
            dpsb.descriptor_count *= ALLOCATION_SIZE;
        }

        let descriptor_set_layouts = vec![descriptor_set_layout; ALLOCATION_SIZE as usize];

        Self {
            template,
            prelude,
            descriptor_set_layouts,
            current_sets: Vec::new(),
            pools: Vec::new(),
        }
    }

    fn allocate_more_sets(&mut self) -> Result<()> {
        // Create descriptor pool of appropriate size
        let create_info = vk::DescriptorPoolCreateInfoBuilder::new()
            .pool_sizes(&self.template)
            .max_sets(ALLOCATION_SIZE); // TODO: Some ops might not need descriptor sets at all! This is potentially wasteful
        let descriptor_pool = unsafe {
            self.prelude
                .device
                .create_descriptor_pool(&create_info, None, None)
        }
        .result()?;

        // Create descriptor sets
        let create_info = vk::DescriptorSetAllocateInfoBuilder::new()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&self.descriptor_set_layouts);
        let descriptor_sets =
            unsafe { self.prelude.device.allocate_descriptor_sets(&create_info) }.result()?;

        self.pools.push(descriptor_pool);
        self.current_sets = descriptor_sets;

        Ok(())
    }

    // My stroop was rather waffle
    /// Get a new descriptor set
    pub fn pop(&mut self) -> Result<vk::DescriptorSet> {
        if let Some(set) = self.current_sets.pop() {
            Ok(set)
        } else {
            self.allocate_more_sets()?;
            Ok(self.current_sets.pop().unwrap())
        }
    }
}

impl Drop for DescriptorSetAllocator {
    fn drop(&mut self) {
        unsafe {
            for pool in self.pools.drain(..) {
                self.prelude.device.destroy_descriptor_pool(Some(pool), None);
            }
        }
    }
}
