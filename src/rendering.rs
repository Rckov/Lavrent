use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use anyrender_vello_hybrid::{ImageCache, ImageManager, VelloHybridScenePainter};
use blitz_html::HtmlDocument;
use vello_gpu::{
    ClearSettings, RenderSettings, RenderSize, RenderTargetConfig, Renderer, Scene, TargetInit,
    TextureBindings,
};
use wgpu::{
    BufferDescriptor, BufferUsages, CommandEncoderDescriptor, DeviceDescriptor, Extent3d, Instance,
    MapMode, Origin3d, PollType, RequestAdapterOptions, TexelCopyBufferInfo, TexelCopyBufferLayout,
    TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureViewDescriptor,
};
use wgpu_context::DeviceHandle;

use crate::{CoreError, HandleError, PowerPreference, RenderTargetHandle};

const TARGET_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
const BYTES_PER_PIXEL: u32 = 4;
const COPY_BYTES_PER_ROW_ALIGNMENT: u32 = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderResult {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rgba8Readback {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

pub(crate) struct RendererState {
    targets: Vec<TargetSlot>,
    gpu: Option<GpuState>,
    power_preference: PowerPreference,
}

struct TargetSlot {
    generation: u32,
    value: Option<OffscreenTarget>,
}

struct OffscreenTarget {
    texture: Texture,
    width: u32,
    height: u32,
    rendered: bool,
}

struct GpuState {
    renderer: Renderer,
    resources: vello_gpu::Resources,
    image_cache: ImageCache,
    device_handle: DeviceHandle,
    device_lost: Arc<AtomicBool>,
    uncaptured_errors: Arc<Mutex<VecDeque<String>>>,
}

impl RendererState {
    pub(crate) const fn new(power_preference: PowerPreference) -> Self {
        Self {
            targets: Vec::new(),
            gpu: None,
            power_preference,
        }
    }

    pub(crate) fn create_target(
        &mut self,
        owner: usize,
        width: u32,
        height: u32,
    ) -> Result<RenderTargetHandle, CoreError> {
        validate_requested_dimensions(width, height)?;
        self.ensure_gpu(width, height)?;
        self.validate_device_dimensions(width, height)?;
        let texture = create_texture(&self.gpu()?.device_handle.device, width, height);
        let target = OffscreenTarget {
            texture,
            width,
            height,
            rendered: false,
        };

        for (index, slot) in self.targets.iter_mut().enumerate() {
            if slot.value.is_none() && slot.generation < u32::MAX {
                slot.generation += 1;
                let generation = slot.generation;
                slot.value = Some(target);
                let slot = u32::try_from(index).map_err(|_| target_capacity_error())?;
                return Ok(RenderTargetHandle::new(owner, slot, generation));
            }
        }

        let slot = u32::try_from(self.targets.len()).map_err(|_| target_capacity_error())?;
        self.targets.push(TargetSlot {
            generation: 1,
            value: Some(target),
        });
        Ok(RenderTargetHandle::new(owner, slot, 1))
    }

    pub(crate) fn resize_target(
        &mut self,
        owner: usize,
        handle: RenderTargetHandle,
        width: u32,
        height: u32,
    ) -> Result<(), CoreError> {
        validate_requested_dimensions(width, height)?;
        let _ = self.target_index(owner, handle)?;
        self.validate_device_dimensions(width, height)?;
        let texture = create_texture(&self.gpu()?.device_handle.device, width, height);
        let target = self.target_mut(owner, handle)?;
        target.texture = texture;
        target.width = width;
        target.height = height;
        target.rendered = false;
        Ok(())
    }

    pub(crate) fn destroy_target(
        &mut self,
        owner: usize,
        handle: RenderTargetHandle,
    ) -> Result<(), CoreError> {
        let index = self.target_index(owner, handle)?;
        let slot = self
            .targets
            .get_mut(index)
            .ok_or(CoreError::Handle(HandleError::Invalid))?;
        if slot.value.take().is_none() {
            return Err(CoreError::Handle(HandleError::Destroyed));
        }
        Ok(())
    }

    pub(crate) fn render(
        &mut self,
        owner: usize,
        handle: RenderTargetHandle,
        document: &mut HtmlDocument,
    ) -> Result<RenderResult, CoreError> {
        let index = self.target_index(owner, handle)?;
        self.check_gpu_state("render")?;
        let (width, height) = {
            let target = self.target(owner, handle)?;
            (target.width, target.height)
        };
        let render_size = RenderSize {
            width: dimension_u16(width)?,
            height: dimension_u16(height)?,
        };

        let gpu = self.gpu_mut()?;
        let depth = Renderer::create_depth_texture_view(&gpu.device_handle.device, &render_size);
        let mut scene = Scene::new_with(
            render_size.width,
            render_size.height,
            RenderSettings::default().level,
        );
        let mut encoder =
            gpu.device_handle
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Lavret render"),
                });
        let image_manager = ImageManager::new(
            &mut gpu.renderer,
            &mut gpu.resources,
            &gpu.device_handle.device,
            &gpu.device_handle.queue,
            &mut encoder,
            &mut gpu.image_cache,
        );
        let mut resource_views = Default::default();
        let mut painter = VelloHybridScenePainter::new(
            &mut scene,
            image_manager,
            &mut resource_views,
            &gpu.device_handle,
        );
        let scale = document.viewport().scale_f64();
        blitz_paint::paint_scene(&mut painter, document, scale, width, height, 0, 0);
        drop(painter);

        let view = self
            .targets
            .get(index)
            .and_then(|slot| slot.value.as_ref())
            .ok_or(CoreError::Handle(HandleError::Destroyed))?
            .texture
            .create_view(&TextureViewDescriptor::default());
        let gpu = self.gpu_mut()?;
        gpu.renderer
            .render(
                &scene,
                &mut gpu.resources,
                &gpu.device_handle.device,
                &gpu.device_handle.queue,
                &mut encoder,
                &render_size,
                &view,
                Some(&depth),
                &TextureBindings::new(),
                TargetInit::Clear(ClearSettings::default()),
            )
            .map_err(|error| CoreError::GpuExecution {
                stage: "render",
                reason: error.to_string(),
            })?;
        ImageManager::new(
            &mut gpu.renderer,
            &mut gpu.resources,
            &gpu.device_handle.device,
            &gpu.device_handle.queue,
            &mut encoder,
            &mut gpu.image_cache,
        )
        .finish_frame();
        gpu.device_handle.queue.submit([encoder.finish()]);
        let _ = gpu.device_handle.device.poll(PollType::Poll);
        self.check_gpu_state("render submission")?;
        self.targets
            .get_mut(index)
            .and_then(|slot| slot.value.as_mut())
            .ok_or(CoreError::Handle(HandleError::Destroyed))?
            .rendered = true;

        Ok(RenderResult { width, height })
    }

    pub(crate) fn readback(
        &mut self,
        owner: usize,
        handle: RenderTargetHandle,
    ) -> Result<Rgba8Readback, CoreError> {
        let target = self.target(owner, handle)?;
        if !target.rendered {
            return Err(CoreError::GpuReadback {
                reason: "render target has no submitted frame".to_owned(),
            });
        }
        let width = target.width;
        let height = target.height;
        self.check_gpu_state("readback")?;
        let row_bytes = width
            .checked_mul(BYTES_PER_PIXEL)
            .ok_or_else(|| readback_size_error(width, height))?;
        let padded_row_bytes = row_bytes
            .checked_add(COPY_BYTES_PER_ROW_ALIGNMENT - 1)
            .map(|value| value / COPY_BYTES_PER_ROW_ALIGNMENT * COPY_BYTES_PER_ROW_ALIGNMENT)
            .ok_or_else(|| readback_size_error(width, height))?;
        let buffer_size = u64::from(padded_row_bytes)
            .checked_mul(u64::from(height))
            .ok_or_else(|| readback_size_error(width, height))?;
        let expected_size = usize::try_from(
            u64::from(row_bytes)
                .checked_mul(u64::from(height))
                .ok_or_else(|| readback_size_error(width, height))?,
        )
        .map_err(|_| readback_size_error(width, height))?;

        let gpu = self.gpu()?;
        let buffer = gpu.device_handle.device.create_buffer(&BufferDescriptor {
            label: Some("Lavret RGBA8 readback"),
            size: buffer_size,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder =
            gpu.device_handle
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Lavret RGBA8 readback copy"),
                });
        encoder.copy_texture_to_buffer(
            TexelCopyTextureInfo {
                texture: &target.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            TexelCopyBufferInfo {
                buffer: &buffer,
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row_bytes),
                    rows_per_image: None,
                },
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        gpu.device_handle.queue.submit([encoder.finish()]);

        let slice = buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        gpu.device_handle
            .device
            .poll(PollType::wait_indefinitely())
            .map_err(|error| CoreError::GpuReadback {
                reason: error.to_string(),
            })?;
        receiver
            .recv()
            .map_err(|error| CoreError::GpuReadback {
                reason: format!("mapping callback failed: {error}"),
            })?
            .map_err(|error| CoreError::GpuReadback {
                reason: error.to_string(),
            })?;

        let mapped = slice
            .get_mapped_range()
            .map_err(|error| CoreError::GpuReadback {
                reason: error.to_string(),
            })?;
        let mut pixels = Vec::with_capacity(expected_size);
        for row in mapped.chunks_exact(padded_row_bytes as usize) {
            pixels.extend_from_slice(&row[..row_bytes as usize]);
        }
        drop(mapped);
        buffer.unmap();
        self.check_gpu_state("readback")?;
        Ok(Rgba8Readback {
            width,
            height,
            pixels,
        })
    }

    fn ensure_gpu(&mut self, width: u32, height: u32) -> Result<(), CoreError> {
        if self.gpu.is_some() {
            return self.check_gpu_state("renderer initialization");
        }

        let instance = Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: match self.power_preference {
                PowerPreference::LowPower => wgpu::PowerPreference::LowPower,
                PowerPreference::HighPerformance => wgpu::PowerPreference::HighPerformance,
            },
            force_fallback_adapter: false,
            compatible_surface: None,
            ..RequestAdapterOptions::default()
        }))
        .map_err(|_| CoreError::AdapterUnavailable)?;
        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
            label: Some("Lavret device"),
            ..DeviceDescriptor::default()
        }))
        .map_err(|error| CoreError::DeviceCreation {
            reason: error.to_string(),
        })?;
        let dimension_limit = device.limits().max_texture_dimension_2d;
        if width > dimension_limit || height > dimension_limit {
            return Err(device_dimension_error(width, height, dimension_limit));
        }

        let device_lost = Arc::new(AtomicBool::new(false));
        let callback_device_lost = Arc::clone(&device_lost);
        device.set_device_lost_callback(move |_, _| {
            callback_device_lost.store(true, Ordering::Release);
        });
        let uncaptured_errors = Arc::new(Mutex::new(VecDeque::new()));
        let callback_errors = Arc::clone(&uncaptured_errors);
        device.on_uncaptured_error(Arc::new(move |error| {
            if let Ok(mut errors) = callback_errors.lock() {
                errors.push_back(error.to_string());
            }
        }));

        let target_config = RenderTargetConfig {
            format: TARGET_FORMAT,
            width: dimension_u16(width)?,
            height: dimension_u16(height)?,
        };
        let (renderer, resources) =
            Renderer::new_with(&device, &target_config, RenderSettings::default());
        self.gpu = Some(GpuState {
            renderer,
            resources,
            image_cache: ImageCache::default(),
            device_handle: DeviceHandle {
                instance,
                adapter,
                device,
                queue,
            },
            device_lost,
            uncaptured_errors,
        });
        Ok(())
    }

    fn validate_device_dimensions(&self, width: u32, height: u32) -> Result<(), CoreError> {
        let limit = self
            .gpu()?
            .device_handle
            .device
            .limits()
            .max_texture_dimension_2d;
        if width > limit || height > limit {
            return Err(device_dimension_error(width, height, limit));
        }
        Ok(())
    }

    fn check_gpu_state(&self, stage: &'static str) -> Result<(), CoreError> {
        let gpu = self.gpu()?;
        if gpu.device_lost.load(Ordering::Acquire) {
            return Err(CoreError::DeviceLost);
        }
        let mut errors = gpu
            .uncaptured_errors
            .lock()
            .map_err(|_| CoreError::GpuExecution {
                stage,
                reason: "GPU error state is unavailable".to_owned(),
            })?;
        if let Some(reason) = errors.pop_front() {
            return Err(CoreError::GpuExecution { stage, reason });
        }
        Ok(())
    }

    fn gpu(&self) -> Result<&GpuState, CoreError> {
        self.gpu.as_ref().ok_or(CoreError::DeviceCreation {
            reason: "renderer is not initialized".to_owned(),
        })
    }

    fn gpu_mut(&mut self) -> Result<&mut GpuState, CoreError> {
        self.gpu.as_mut().ok_or(CoreError::DeviceCreation {
            reason: "renderer is not initialized".to_owned(),
        })
    }

    fn target(
        &self,
        owner: usize,
        handle: RenderTargetHandle,
    ) -> Result<&OffscreenTarget, CoreError> {
        let index = self.target_index(owner, handle)?;
        self.targets
            .get(index)
            .and_then(|slot| slot.value.as_ref())
            .ok_or(CoreError::Handle(HandleError::Destroyed))
    }

    fn target_mut(
        &mut self,
        owner: usize,
        handle: RenderTargetHandle,
    ) -> Result<&mut OffscreenTarget, CoreError> {
        let index = self.target_index(owner, handle)?;
        self.targets
            .get_mut(index)
            .and_then(|slot| slot.value.as_mut())
            .ok_or(CoreError::Handle(HandleError::Destroyed))
    }

    fn target_index(&self, owner: usize, handle: RenderTargetHandle) -> Result<usize, CoreError> {
        if handle.owner != owner {
            return Err(CoreError::Handle(HandleError::Foreign));
        }
        let index =
            usize::try_from(handle.slot).map_err(|_| CoreError::Handle(HandleError::Invalid))?;
        let slot = self
            .targets
            .get(index)
            .ok_or(CoreError::Handle(HandleError::Invalid))?;
        if slot.generation != handle.generation {
            return Err(CoreError::Handle(HandleError::Stale));
        }
        Ok(index)
    }
}

fn validate_requested_dimensions(width: u32, height: u32) -> Result<(), CoreError> {
    if width == 0 || height == 0 {
        return Err(CoreError::InvalidArgument {
            name: "render target dimensions",
            reason: "width and height must be positive".to_owned(),
        });
    }
    let _ = dimension_u16(width)?;
    let _ = dimension_u16(height)?;
    Ok(())
}

fn dimension_u16(value: u32) -> Result<u16, CoreError> {
    u16::try_from(value).map_err(|_| CoreError::ResourceLimit {
        resource: format!("render target dimension {value}"),
        limit: "Vello GPU u16 render dimension".to_owned(),
    })
}

fn create_texture(device: &wgpu::Device, width: u32, height: u32) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some("Lavret offscreen target"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TARGET_FORMAT,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn target_capacity_error() -> CoreError {
    CoreError::ResourceLimit {
        resource: "render target".to_owned(),
        limit: "Lavret render-target handle capacity".to_owned(),
    }
}

fn device_dimension_error(width: u32, height: u32, limit: u32) -> CoreError {
    CoreError::ResourceLimit {
        resource: format!("render target {width}x{height}"),
        limit: format!("GPU max_texture_dimension_2d={limit}"),
    }
}

fn readback_size_error(width: u32, height: u32) -> CoreError {
    CoreError::ResourceLimit {
        resource: format!("RGBA8 readback {width}x{height}"),
        limit: "addressable tightly packed pixel buffer".to_owned(),
    }
}
