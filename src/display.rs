//! Display module - Hardware-accelerated rendering with pixels + winit + egui
//!
//! This module manages the CYD's 320x240 RGB565 framebuffer and renders it
//! using the pixels crate for hardware acceleration, with egui overlay for UI.

use pixels::{Pixels, SurfaceTexture};
use winit::window::Window;
use winit::event::WindowEvent;
use parking_lot::Mutex;
use std::sync::Arc;
use anyhow::Result;

pub const DISPLAY_WIDTH: usize = 320;
pub const DISPLAY_HEIGHT: usize = 240;
pub const FRAMEBUFFER_SIZE: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

/// Display manager with hardware-accelerated rendering
pub struct Display {
    /// Shared framebuffer (RGB565 format)
    framebuffer: Arc<Mutex<Vec<u16>>>,
    /// Hardware-accelerated pixel buffer
    pixels: Pixels,
    /// egui context and state
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    /// Dirty flag to avoid unnecessary renders
    dirty: bool,
}

impl Display {
    /// Create a new display bound to the given window
    pub fn new(window: &Window) -> Result<Self> {
        let window_size = window.inner_size();

        // Create surface texture for the window
        let surface_texture = SurfaceTexture::new(
            window_size.width,
            window_size.height,
            window,
        );

        // Create pixels buffer to match display size (320x240)
        // It will be scaled to fit the window
        let pixels = Pixels::new(
            DISPLAY_WIDTH as u32,
            DISPLAY_HEIGHT as u32,
            surface_texture,
        )?;

        // Initialize framebuffer (all black)
        let framebuffer = Arc::new(Mutex::new(vec![0u16; FRAMEBUFFER_SIZE]));

        // Initialize egui
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(window);

        // Create egui renderer
        let egui_renderer = egui_wgpu::Renderer::new(
            pixels.device(),
            pixels.render_texture_format(),
            None,
            1,
        );

        Ok(Self {
            framebuffer,
            pixels,
            egui_ctx,
            egui_state,
            egui_renderer,
            dirty: true,
        })
    }

    /// Get a clone of the framebuffer Arc for sharing with emulator
    pub fn framebuffer(&self) -> Arc<Mutex<Vec<u16>>> {
        self.framebuffer.clone()
    }

    /// Mark display as dirty (needs re-render)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Render the framebuffer to the screen (without egui)
    ///
    /// Converts RGB565 -> RGBA8888 and renders via pixels
    /// Call render_with_ui() instead to include egui overlay
    pub fn render(&mut self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }

        let fb = self.framebuffer.lock();
        let frame = self.pixels.frame_mut();

        // Convert RGB565 -> RGBA8888
        for (i, &pixel) in fb.iter().enumerate() {
            // Extract RGB565 components
            let r5 = ((pixel >> 11) & 0x1F) as u8;
            let g6 = ((pixel >> 5) & 0x3F) as u8;
            let b5 = (pixel & 0x1F) as u8;

            // Expand to 8 bits
            let r8 = (r5 << 3) | (r5 >> 2);
            let g8 = (g6 << 2) | (g6 >> 4);
            let b8 = (b5 << 3) | (b5 >> 2);

            // Write to RGBA frame buffer
            let rgba_idx = i * 4;
            frame[rgba_idx] = r8;
            frame[rgba_idx + 1] = g8;
            frame[rgba_idx + 2] = b8;
            frame[rgba_idx + 3] = 255;
        }

        drop(fb); // Release lock before rendering

        // Render to screen
        self.pixels.render()?;
        self.dirty = false;

        Ok(())
    }

    /// Render the framebuffer with egui UI overlay
    ///
    /// WORKAROUND: pixels.render_with() doesn't work correctly with egui overlay,
    /// so we render pixels first, then manually overlay egui using wgpu directly
    pub fn render_with_ui<F>(&mut self, window: &Window, ui_fn: F) -> Result<()>
    where
        F: FnOnce(&egui::Context),
    {
        // Step 1: Convert RGB565 framebuffer to RGBA8888
        {
            let fb = self.framebuffer.lock();
            let frame = self.pixels.frame_mut();

            // Convert RGB565 -> RGBA8888
            for (i, &pixel) in fb.iter().enumerate() {
                let r5 = ((pixel >> 11) & 0x1F) as u8;
                let g6 = ((pixel >> 5) & 0x3F) as u8;
                let b5 = (pixel & 0x1F) as u8;

                let r8 = (r5 << 3) | (r5 >> 2);
                let g8 = (g6 << 2) | (g6 >> 4);
                let b8 = (b5 << 3) | (b5 >> 2);

                let rgba_idx = i * 4;
                frame[rgba_idx] = r8;
                frame[rgba_idx + 1] = g8;
                frame[rgba_idx + 2] = b8;
                frame[rgba_idx + 3] = 255;
            }
        }

        // Step 2: Render pixels texture using render_with, and add egui on top
        let raw_input = self.egui_state.take_egui_input(window);
        let full_output = self.egui_ctx.run(raw_input, ui_fn);

        self.egui_state.handle_platform_output(window, &self.egui_ctx, full_output.platform_output);

        let clipped_primitives = self.egui_ctx.tessellate(full_output.shapes);
        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [window.inner_size().width, window.inner_size().height],
            pixels_per_point: window.scale_factor() as f32,
        };

        // Render with pixels, adding egui overlay
        self.pixels.render_with(|encoder, render_target, context| {
            // CRITICAL: First render the pixels scaling pass
            // This converts the pixel buffer to the screen
            context.scaling_renderer.render(encoder, render_target);

            // Then update egui textures
            for (id, image_delta) in &full_output.textures_delta.set {
                self.egui_renderer.update_texture(&context.device, &context.queue, *id, image_delta);
            }

            // Update egui buffers
            self.egui_renderer.update_buffers(
                &context.device,
                &context.queue,
                encoder,
                &clipped_primitives,
                &screen_descriptor,
            );

            // Render egui on top with LoadOp::Load to preserve pixels
            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("egui"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: render_target,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,  // Preserve pixels texture
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                self.egui_renderer.render(&mut rpass, &clipped_primitives, &screen_descriptor);
            }

            // Cleanup
            for id in &full_output.textures_delta.free {
                self.egui_renderer.free_texture(id);
            }

            Ok(())
        })?;

        self.dirty = false;
        Ok(())
    }

    /// Handle window resize events
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        self.pixels.resize_surface(width, height)?;
        // egui automatically handles resize through the state
        self.mark_dirty();
        Ok(())
    }

    /// Get the pixels object (for egui integration if needed)
    pub fn pixels_mut(&mut self) -> &mut Pixels {
        &mut self.pixels
    }

    /// Handle winit events for egui
    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        let response = self.egui_state.on_event(&self.egui_ctx, event);
        response.consumed
    }

    /// Fill framebuffer with a solid color (for testing)
    #[allow(dead_code)]
    pub fn fill(&self, color: u16) {
        let mut fb = self.framebuffer.lock();
        fb.fill(color);
    }

    /// Draw a test pattern (for debugging)
    #[allow(dead_code)]
    pub fn test_pattern(&self) {
        let mut fb = self.framebuffer.lock();
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                // Create color bars
                let r = ((x * 32) / DISPLAY_WIDTH) as u16;
                let g = ((y * 64) / DISPLAY_HEIGHT) as u16;
                let b = (((x + y) * 32) / (DISPLAY_WIDTH + DISPLAY_HEIGHT)) as u16;

                let rgb565 = (r << 11) | (g << 5) | b;
                fb[y * DISPLAY_WIDTH + x] = rgb565;
            }
        }
    }
}

/// RGB565 color constants
#[allow(dead_code)]
pub mod colors {
    pub const BLACK: u16 = 0x0000;
    pub const WHITE: u16 = 0xFFFF;
    pub const RED: u16 = 0xF800;
    pub const GREEN: u16 = 0x07E0;
    pub const BLUE: u16 = 0x001F;
    pub const YELLOW: u16 = 0xFFE0;
    pub const CYAN: u16 = 0x07FF;
    pub const MAGENTA: u16 = 0xF81F;
    pub const GRAY: u16 = 0x7BEF;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb565_conversion() {
        // Test pure red (0b11111_000000_00000)
        let red = 0xF800u16;
        let r5 = ((red >> 11) & 0x1F) as u8;
        let g6 = ((red >> 5) & 0x3F) as u8;
        let b5 = (red & 0x1F) as u8;

        assert_eq!(r5, 31);
        assert_eq!(g6, 0);
        assert_eq!(b5, 0);

        let r8 = (r5 << 3) | (r5 >> 2);
        assert_eq!(r8, 255);
    }

    #[test]
    fn test_framebuffer_size() {
        assert_eq!(FRAMEBUFFER_SIZE, 76800);
    }
}
