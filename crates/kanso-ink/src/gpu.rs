//! Feature-gated offscreen `wgpu` renderer.
//!
//! Enabled only when the `gpu` Cargo feature is active:
//!
//! ```toml
//! kanso-ink = { path = "…", features = ["gpu"] }
//! ```
//!
//! The public entry-point is [`render_rgba`], which tessellates a [`SketchDoc`]
//! and renders it to a flat `width × height` RGBA byte buffer. It returns
//! `None` when no GPU adapter is available (expected in headless CI).

#![cfg(feature = "gpu")]

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::format::SketchDoc;
use crate::tessellate;

// ─── GPU vertex ──────────────────────────────────────────────────────────────

/// Mirrors [`tessellate::Vertex`] but carries the `Pod + Zeroable` bounds
/// required by `bytemuck` for safe buffer uploads.
///
/// Positions are pre-converted to NDC on the CPU so the WGSL shader stays
/// trivial (pass-through).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuVertex {
    /// Normalised device coordinates: x ∈ [−1, 1], y ∈ [−1, 1].
    pos: [f32; 2],
    /// RGBA in 0.0 – 1.0.
    color: [f32; 4],
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Render `doc` to a flat RGBA byte buffer of size `width × height`.
///
/// Returns `None` if:
/// - no GPU adapter is available (headless / CI environment), or
/// - `width` or `height` is zero.
///
/// The returned `Vec<u8>` is `width * height * 4` bytes, row-major,
/// top-left origin, `Rgba8UnormSrgb` encoding.
pub fn render_rgba(doc: &SketchDoc, width: u32, height: u32) -> Option<Vec<u8>> {
    pollster::block_on(render_rgba_async(doc, width, height))
}

// ─── Async implementation ─────────────────────────────────────────────────────

async fn render_rgba_async(doc: &SketchDoc, width: u32, height: u32) -> Option<Vec<u8>> {
    if width == 0 || height == 0 {
        return None;
    }

    // ── Adapter / device ──────────────────────────────────────────────────────

    let instance = wgpu::Instance::default();

    // `request_adapter` returns `Result<Adapter, _>` in wgpu 29.x — degrade to
    // `None` when no adapter is available (headless CI).
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::None,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .ok()?;

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("kanso-ink offscreen"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        })
        .await
        .ok()?;

    // ── Tessellate & convert to NDC ───────────────────────────────────────────

    let cpu_mesh = tessellate::tessellate_doc(doc);

    let gpu_verts: Vec<GpuVertex> = cpu_mesh
        .vertices
        .iter()
        .map(|v| GpuVertex {
            pos: pixel_to_ndc(v.pos[0], v.pos[1], width, height),
            color: v.color,
        })
        .collect();

    // ── Buffers ───────────────────────────────────────────────────────────────

    let has_geometry = !gpu_verts.is_empty();

    let vertex_buffer = if has_geometry {
        Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("kanso vertex buffer"),
                contents: bytemuck::cast_slice(&gpu_verts),
                usage: wgpu::BufferUsages::VERTEX,
            }),
        )
    } else {
        None
    };

    let index_buffer = if has_geometry {
        Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("kanso index buffer"),
                contents: bytemuck::cast_slice(&cpu_mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            }),
        )
    } else {
        None
    };

    let index_count = cpu_mesh.indices.len() as u32;

    // ── Render target ─────────────────────────────────────────────────────────

    let texture_format = wgpu::TextureFormat::Rgba8UnormSrgb;

    let render_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("kanso render target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: texture_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    let render_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());

    // ── Shader ────────────────────────────────────────────────────────────────

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("kanso stroke shader"),
        source: wgpu::ShaderSource::Wgsl(WGSL_SHADER.into()),
    });

    // ── Pipeline ──────────────────────────────────────────────────────────────

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("kanso pipeline layout"),
        bind_group_layouts: &[],
        immediate_size: 0,
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("kanso render pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<GpuVertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4],
            }],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: texture_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None, // No culling: strokes are thin and winding can vary.
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    // ── Readback buffer ───────────────────────────────────────────────────────

    // wgpu requires buffer rows to be aligned to COPY_BYTES_PER_ROW_ALIGNMENT.
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let unpadded_row = width * 4;
    let padded_row = (unpadded_row + align - 1) / align * align;

    let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("kanso readback buffer"),
        size: (padded_row * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    // ── Render pass ───────────────────────────────────────────────────────────

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("kanso encoder"),
    });

    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("kanso render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &render_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        if has_geometry {
            let vb = vertex_buffer.as_ref().unwrap();
            let ib = index_buffer.as_ref().unwrap();
            pass.set_pipeline(&render_pipeline);
            pass.set_vertex_buffer(0, vb.slice(..));
            pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..index_count, 0, 0..1);
        }
    }

    // ── Copy texture → buffer ─────────────────────────────────────────────────

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &render_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    queue.submit(std::iter::once(encoder.finish()));

    // ── Map & unpad rows ─────────────────────────────────────────────────────

    let buffer_slice = readback_buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });

    // Poll until the GPU work is done and the map completes.
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });

    rx.recv().ok()?.ok()?;

    let mapped = buffer_slice.get_mapped_range();
    let padded_data: &[u8] = &mapped;

    // Strip the row padding so callers receive exactly `width * height * 4` bytes.
    let mut pixels = Vec::with_capacity((unpadded_row * height) as usize);
    for row in 0..height as usize {
        let start = row * padded_row as usize;
        let end = start + unpadded_row as usize;
        pixels.extend_from_slice(&padded_data[start..end]);
    }

    drop(mapped);
    readback_buffer.unmap();

    Some(pixels)
}

// ─── NDC helper ───────────────────────────────────────────────────────────────

/// Convert canvas-pixel coordinates (origin top-left) to NDC (origin centre,
/// y grows upward).
#[inline]
fn pixel_to_ndc(px: f32, py: f32, width: u32, height: u32) -> [f32; 2] {
    let ndc_x = px / width as f32 * 2.0 - 1.0;
    let ndc_y = 1.0 - py / height as f32 * 2.0;
    [ndc_x, ndc_y]
}

// ─── WGSL shader source ───────────────────────────────────────────────────────

/// Minimal pass-through shader: vertex receives pre-converted NDC + colour,
/// fragment just outputs the interpolated colour.
const WGSL_SHADER: &str = r#"
struct VertexIn {
    @location(0) pos:   vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0)       color:    vec4<f32>,
};

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_pos = vec4<f32>(in.pos, 0.0, 1.0);
    out.color    = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return in.color;
}
"#;
