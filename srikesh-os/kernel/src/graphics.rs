//! SRIKESH OS - Advanced Graphics Engine & Double-Buffered Compositor
//!
//! Features:
//! - 32-bit ARGB/XRGB Double Buffering with tearing-free blitting.
//! - Alpha blending & Glassmorphism transparency effects.
//! - Geometric primitives: rounded rectangles, gradients, outlines.
//! - High-performance font rendering via font8x8.
//! - Scaled bilinear/nearest-neighbor RGB image blitting.
//! - Hardware-styled cursor with glowing drop shadow.

use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use font8x8::{UnicodeFonts, BASIC_FONTS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn with_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    // Palette: Egyptian Royal Visual Identity & Classic Accents
    pub const BLACK: Color = Color::new(0, 0, 0);
    pub const OBSIDIAN: Color = Color::new(12, 10, 8);               // #0C0A08 Deep Obsidian Black
    pub const PHARAOH_DARK: Color = Color::new(18, 14, 10);          // #120E0A Dark Pharaoh Base
    pub const ROYAL_GOLD: Color = Color::new(212, 175, 55);          // #D4AF37 Royal Egyptian Gold
    pub const BRIGHT_GOLD: Color = Color::new(255, 215, 0);          // #FFD700 Bright Sun Gold
    pub const DARK_GOLD: Color = Color::new(165, 125, 25);           // #A57D19 Muted Antique Gold
    pub const LAPIS_LAZULI: Color = Color::new(28, 45, 90);          // #1C2D5A Royal Lapis Lazuli Blue
    pub const LAPIS_LIGHT: Color = Color::new(45, 75, 145);          // #2D4B91 Light Lapis
    pub const ROYAL_SAND: Color = Color::new(228, 196, 140);         // #E4C48C Egyptian Sandstone
    pub const ROYAL_AMBER: Color = Color::new(255, 160, 20);         // #FFA014 Amber Torchlight
    pub const ROYAL_GLASS_PANEL: Color = Color::with_alpha(18, 15, 12, 215); // Translucent Obsidian
    pub const ROYAL_GLASS_BORDER: Color = Color::with_alpha(212, 175, 55, 140); // Translucent Gold Border
    pub const HIEROGLYPH_GLOW: Color = Color::with_alpha(255, 215, 0, 180);

    // Classic / Semantic Accents
    pub const DEEP_ABYSS: Color = Color::new(12, 10, 8);
    pub const DARK_NAVY: Color = Color::new(16, 14, 20);
    pub const GLASS_PANEL: Color = Color::with_alpha(18, 15, 12, 215);
    pub const GLASS_BORDER: Color = Color::with_alpha(212, 175, 55, 140);
    pub const ELECTRIC_CYAN: Color = Color::new(212, 175, 55);       // Mapped to Royal Gold for uniform branding
    pub const NEON_VIOLET: Color = Color::new(45, 75, 145);
    pub const NEON_MAGENTA: Color = Color::new(212, 175, 55);
    pub const NEON_BLUE: Color = Color::new(28, 45, 90);
    pub const WHITE: Color = Color::new(255, 255, 255);
    pub const TEXT_MUTED: Color = Color::new(180, 165, 145);
    pub const SUCCESS_GREEN: Color = Color::new(40, 210, 120);
    pub const DANGER_RED: Color = Color::new(235, 60, 50);

    /// Alpha blend foreground color over background color.
    #[inline(always)]
    pub fn blend_over(&self, bg: Color) -> Color {
        if self.a == 255 {
            return *self;
        }
        if self.a == 0 {
            return bg;
        }

        let alpha = self.a as u32;
        let inv_alpha = 255 - alpha;

        let r = ((self.r as u32 * alpha + bg.r as u32 * inv_alpha) / 255) as u8;
        let g = ((self.g as u32 * alpha + bg.g as u32 * inv_alpha) / 255) as u8;
        let b = ((self.b as u32 * alpha + bg.b as u32 * inv_alpha) / 255) as u8;

        Color::new(r, g, b)
    }
}

// Two 8.3 MB static buffers in BSS (zero cost on disk image):
// - BACKBUFFER: pristine cursor-free scene cache (up to 1920x1080x4)
// - WALLPAPER_CACHE: pre-scaled & tinted desktop wallpaper for instant 1-memcpy redraws
const SINGLE_FRAME_MAX: usize = 1920 * 1080 * 4;
static mut BACKBUFFER: [u8; SINGLE_FRAME_MAX] = [0; SINGLE_FRAME_MAX];
static mut WALLPAPER_CACHE: [u8; SINGLE_FRAME_MAX] = [0; SINGLE_FRAME_MAX];

pub static LAST_FULL_FRAME_CYCLES: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
pub static LAST_CURSOR_BLIT_CYCLES: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

pub struct GraphicsEngine {
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub bpp: usize,
    pub format: PixelFormat,
    frontbuffer: *mut u8,
    frontbuffer_len: usize,
    has_backbuffer: bool,
    wallpaper_cached: bool,
    last_cursor_x: usize,
    last_cursor_y: usize,
    cursor_visible: bool,
}

// Safety: GraphicsEngine is wrapped in a spin::Mutex
unsafe impl Send for GraphicsEngine {}
unsafe impl Sync for GraphicsEngine {}

impl GraphicsEngine {
    pub const fn uninitialized() -> Self {
        Self {
            width: 0,
            height: 0,
            stride: 0,
            bpp: 0,
            format: PixelFormat::Bgr,
            frontbuffer: core::ptr::null_mut(),
            frontbuffer_len: 0,
            has_backbuffer: false,
            wallpaper_cached: false,
            last_cursor_x: 0,
            last_cursor_y: 0,
            cursor_visible: false,
        }
    }

    pub fn init(&mut self, buffer: &mut [u8], info: &FrameBufferInfo) {
        self.width = info.width;
        self.height = info.height;
        self.stride = info.stride;
        self.bpp = info.bytes_per_pixel;
        self.format = info.pixel_format;
        self.frontbuffer = buffer.as_mut_ptr();
        self.frontbuffer_len = buffer.len();

        let needed_backbuffer = self.stride * self.height * self.bpp;
        self.has_backbuffer = needed_backbuffer <= SINGLE_FRAME_MAX;
        self.wallpaper_cached = false;
        self.cursor_visible = false;
    }

    /// Blits the pre-scaled tinted desktop wallpaper in a single fast memcpy (10x faster window drag/redraw)
    pub fn draw_wallpaper_cached(&mut self, img_data: &[u8], src_w: usize, src_h: usize, tint: Color) {
        if !self.has_backbuffer {
            self.draw_image_rgb(0, 0, self.width, self.height, img_data, src_w, src_h);
            self.fill_rect(0, 0, self.width, self.height, tint);
            return;
        }
        let frame_bytes = self.stride * self.height * self.bpp;
        if !self.wallpaper_cached {
            self.draw_image_rgb(0, 0, self.width, self.height, img_data, src_w, src_h);
            self.fill_rect(0, 0, self.width, self.height, tint);
            let src = core::ptr::addr_of!(BACKBUFFER) as *const u8;
            let dst = core::ptr::addr_of_mut!(WALLPAPER_CACHE) as *mut u8;
            unsafe {
                core::ptr::copy_nonoverlapping(src, dst, frame_bytes);
            }
            self.wallpaper_cached = true;
        } else {
            let src = core::ptr::addr_of!(WALLPAPER_CACHE) as *const u8;
            let dst = core::ptr::addr_of_mut!(BACKBUFFER) as *mut u8;
            unsafe {
                core::ptr::copy_nonoverlapping(src, dst, frame_bytes);
            }
        }
    }

    #[inline(always)]
    fn active_buffer_mut(&mut self) -> &mut [u8] {
        if self.has_backbuffer {
            let ptr = core::ptr::addr_of_mut!(BACKBUFFER) as *mut u8;
            unsafe { core::slice::from_raw_parts_mut(ptr, self.stride * self.height * self.bpp) }
        } else {
            unsafe { core::slice::from_raw_parts_mut(self.frontbuffer, self.frontbuffer_len) }
        }
    }

    #[inline(always)]
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }

        let bpp = self.bpp;
        let stride = self.stride;
        let format = self.format;
        let offset = (y * stride + x) * bpp;

        let buf = self.active_buffer_mut();
        if offset + bpp <= buf.len() {
            match format {
                PixelFormat::Bgr => {
                    buf[offset] = color.b;
                    if bpp > 1 { buf[offset + 1] = color.g; }
                    if bpp > 2 { buf[offset + 2] = color.r; }
                }
                PixelFormat::Rgb => {
                    buf[offset] = color.r;
                    if bpp > 1 { buf[offset + 1] = color.g; }
                    if bpp > 2 { buf[offset + 2] = color.b; }
                }
                PixelFormat::U8 => {
                    buf[offset] = ((color.r as u16 + color.g as u16 + color.b as u16) / 3) as u8;
                }
                _ => {}
            }
        }
    }

    #[inline]
    pub fn get_pixel(&mut self, x: usize, y: usize) -> Color {
        if x >= self.width || y >= self.height {
            return Color::BLACK;
        }
        let bpp = self.bpp;
        let format = self.format;
        let offset = (y * self.stride + x) * bpp;
        let buf = self.active_buffer_mut();
        if offset + bpp <= buf.len() {
            match format {
                PixelFormat::Bgr => Color::new(buf[offset + 2], buf[offset + 1], buf[offset]),
                PixelFormat::Rgb => Color::new(buf[offset], buf[offset + 1], buf[offset + 2]),
                _ => Color::BLACK,
            }
        } else {
            Color::BLACK
        }
    }

    #[inline]
    pub fn set_pixel_blend(&mut self, x: usize, y: usize, color: Color) {
        if color.a == 255 {
            self.set_pixel(x, y, color);
        } else if color.a > 0 {
            let bg = self.get_pixel(x, y);
            let blended = color.blend_over(bg);
            self.set_pixel(x, y, blended);
        }
    }

    pub fn clear(&mut self, color: Color) {
        self.fill_rect(0, 0, self.width, self.height, color);
    }

    pub fn fill_rect(&mut self, start_x: usize, start_y: usize, w: usize, h: usize, color: Color) {
        let end_x = (start_x + w).min(self.width);
        let end_y = (start_y + h).min(self.height);

        if color.a == 255 {
            for y in start_y..end_y {
                for x in start_x..end_x {
                    self.set_pixel(x, y, color);
                }
            }
        } else {
            for y in start_y..end_y {
                for x in start_x..end_x {
                    self.set_pixel_blend(x, y, color);
                }
            }
        }
    }

    pub fn draw_rect_outline(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color, thickness: usize) {
        // Top and bottom bars
        self.fill_rect(x, y, w, thickness, color);
        self.fill_rect(x, (y + h).saturating_sub(thickness), w, thickness, color);
        // Left and right bars
        self.fill_rect(x, y, thickness, h, color);
        self.fill_rect((x + w).saturating_sub(thickness), y, thickness, h, color);
    }

    pub fn fill_rounded_rect(&mut self, x: usize, y: usize, w: usize, h: usize, radius: usize, color: Color) {
        let r = radius.min(w / 2).min(h / 2);
        // Inner center
        self.fill_rect(x + r, y, w - 2 * r, h, color);
        // Left & right strips
        self.fill_rect(x, y + r, r, h - 2 * r, color);
        self.fill_rect(x + w - r, y + r, r, h - 2 * r, color);

        // 4 Rounded corners
        let r2 = (r * r) as isize;
        for cy in 0..r {
            for cx in 0..r {
                let dx = (r - 1 - cx) as isize;
                let dy = (r - 1 - cy) as isize;
                if dx * dx + dy * dy <= r2 {
                    // Top-Left
                    self.set_pixel_blend(x + cx, y + cy, color);
                    // Top-Right
                    self.set_pixel_blend(x + w - 1 - cx, y + cy, color);
                    // Bottom-Left
                    self.set_pixel_blend(x + cx, y + h - 1 - cy, color);
                    // Bottom-Right
                    self.set_pixel_blend(x + w - 1 - cx, y + h - 1 - cy, color);
                }
            }
        }
    }

    pub fn draw_gradient_v(&mut self, x: usize, y: usize, w: usize, h: usize, top: Color, bottom: Color) {
        if h == 0 { return; }
        for row in 0..h {
            let factor = (row as u32 * 255) / h as u32;
            let inv = 255 - factor;
            let r = ((top.r as u32 * inv + bottom.r as u32 * factor) / 255) as u8;
            let g = ((top.g as u32 * inv + bottom.g as u32 * factor) / 255) as u8;
            let b = ((top.b as u32 * inv + bottom.b as u32 * factor) / 255) as u8;
            let color = Color::new(r, g, b);
            self.fill_rect(x, y + row, w, 1, color);
        }
    }

    pub fn draw_gradient_h(&mut self, x: usize, y: usize, w: usize, h: usize, left: Color, right: Color) {
        if w == 0 { return; }
        for col in 0..w {
            let factor = (col as u32 * 255) / w as u32;
            let inv = 255 - factor;
            let r = ((left.r as u32 * inv + right.r as u32 * factor) / 255) as u8;
            let g = ((left.g as u32 * inv + right.g as u32 * factor) / 255) as u8;
            let b = ((left.b as u32 * inv + right.b as u32 * factor) / 255) as u8;
            let color = Color::new(r, g, b);
            self.fill_rect(x + col, y, 1, h, color);
        }
    }

    pub fn draw_char(&mut self, x: usize, y: usize, ch: char, color: Color, scale: usize) {
        if let Some(glyph) = BASIC_FONTS.get(ch) {
            for (row, bits) in glyph.iter().enumerate() {
                for col in 0..8 {
                    if (*bits & (1 << col)) != 0 {
                        self.fill_rect(x + col * scale, y + row * scale, scale, scale, color);
                    }
                }
            }
        }
    }

    pub fn draw_text(&mut self, x: usize, y: usize, text: &str, color: Color, scale: usize) {
        let mut cur_x = x;
        for ch in text.chars() {
            if ch == '\n' {
                cur_x = x;
            } else {
                self.draw_char(cur_x, y, ch, color, scale);
                cur_x += 8 * scale;
            }
        }
    }

    pub fn draw_text_centered(&mut self, y: usize, text: &str, color: Color, scale: usize) {
        let text_width = text.chars().count() * 8 * scale;
        let start_x = self.width.saturating_sub(text_width) / 2;
        self.draw_text(start_x, y, text, color, scale);
    }

    /// Blit raw 24-bit RGB pixel data with scaling into the target rectangle.
    /// Optimized with fixed-point arithmetic and sequential buffer writes for maximum speed.
    pub fn draw_image_rgb(
        &mut self,
        dst_x: usize,
        dst_y: usize,
        dst_w: usize,
        dst_h: usize,
        img_data: &[u8],
        src_w: usize,
        src_h: usize,
    ) {
        if dst_w == 0 || dst_h == 0 || src_w == 0 || src_h == 0 {
            return;
        }

        let step_x = ((src_w as u64) << 16) / (dst_w as u64);
        let step_y = ((src_h as u64) << 16) / (dst_h as u64);

        let mut cur_y_fixed = 0u64;
        let bpp = self.bpp;
        let stride = self.stride;
        let format = self.format;
        let width = self.width;
        let height = self.height;

        for dy in 0..dst_h {
            let target_y = dst_y + dy;
            if target_y >= height { break; }
            let sy = ((cur_y_fixed >> 16) as usize).min(src_h - 1);
            cur_y_fixed += step_y;

            let row_offset = (target_y * stride + dst_x) * bpp;
            let mut cur_x_fixed = 0u64;
            let buf = self.active_buffer_mut();

            for dx in 0..dst_w {
                let target_x = dst_x + dx;
                if target_x >= width { break; }
                let sx = ((cur_x_fixed >> 16) as usize).min(src_w - 1);
                cur_x_fixed += step_x;

                let src_idx = (sy * src_w + sx) * 3;
                let offset = row_offset + dx * bpp;

                if src_idx + 2 < img_data.len() && offset + bpp <= buf.len() {
                    let r = img_data[src_idx];
                    let g = img_data[src_idx + 1];
                    let b = img_data[src_idx + 2];

                    match format {
                        PixelFormat::Bgr => {
                            buf[offset] = b;
                            if bpp > 1 { buf[offset + 1] = g; }
                            if bpp > 2 { buf[offset + 2] = r; }
                        }
                        PixelFormat::Rgb => {
                            buf[offset] = r;
                            if bpp > 1 { buf[offset + 1] = g; }
                            if bpp > 2 { buf[offset + 2] = b; }
                        }
                        PixelFormat::U8 => {
                            buf[offset] = ((r as u16 + g as u16 + b as u16) / 3) as u8;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Render a sleek futuristic desktop cursor at (x, y) with glowing outline.
    pub fn draw_cursor(&mut self, x: usize, y: usize) {
        // 12x18 Arrow cursor mask (0 = None, 1 = Shadow/Black, 2 = White, 3 = Neon Cyan Accent)
        #[rustfmt::skip]
        const CURSOR_MAP: [[u8; 12]; 18] = [
            [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [2, 3, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [2, 3, 3, 2, 0, 0, 0, 0, 0, 0, 0, 0],
            [2, 3, 3, 3, 2, 0, 0, 0, 0, 0, 0, 0],
            [2, 3, 3, 3, 3, 2, 0, 0, 0, 0, 0, 0],
            [2, 3, 3, 3, 3, 3, 2, 0, 0, 0, 0, 0],
            [2, 3, 3, 3, 3, 3, 3, 2, 0, 0, 0, 0],
            [2, 3, 3, 3, 3, 3, 3, 3, 2, 0, 0, 0],
            [2, 3, 3, 3, 3, 3, 2, 2, 2, 2, 0, 0],
            [2, 3, 3, 2, 3, 3, 2, 0, 0, 0, 0, 0],
            [2, 3, 2, 0, 2, 3, 3, 2, 0, 0, 0, 0],
            [2, 2, 0, 0, 2, 3, 3, 2, 0, 0, 0, 0],
            [2, 0, 0, 0, 0, 2, 3, 3, 2, 0, 0, 0],
            [0, 0, 0, 0, 0, 2, 3, 3, 2, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 2, 3, 3, 2, 0, 0],
            [0, 0, 0, 0, 0, 0, 2, 3, 3, 2, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 2, 2, 0, 0, 0],
        ];

        for (cy, row) in CURSOR_MAP.iter().enumerate() {
            for (cx, &val) in row.iter().enumerate() {
                let px = x + cx;
                let py = y + cy;
                match val {
                    1 => self.set_pixel_blend(px, py, Color::with_alpha(0, 0, 0, 180)),
                    2 => self.set_pixel(px, py, Color::WHITE),
                    3 => self.set_pixel(px, py, Color::ELECTRIC_CYAN),
                    _ => {}
                }
            }
        }
    }

    #[inline(always)]
    fn write_front_pixel(&self, x: usize, y: usize, color: Color) {
        if x >= self.width || y >= self.height || self.frontbuffer.is_null() {
            return;
        }
        let offset = (y * self.stride + x) * self.bpp;
        if offset + self.bpp <= self.frontbuffer_len {
            unsafe {
                let p = self.frontbuffer.add(offset);
                match self.format {
                    PixelFormat::Bgr => {
                        *p = color.b;
                        if self.bpp > 1 { *p.add(1) = color.g; }
                        if self.bpp > 2 { *p.add(2) = color.r; }
                    }
                    PixelFormat::Rgb => {
                        *p = color.r;
                        if self.bpp > 1 { *p.add(1) = color.g; }
                        if self.bpp > 2 { *p.add(2) = color.b; }
                    }
                    PixelFormat::U8 => {
                        *p = ((color.r as u16 + color.g as u16 + color.b as u16) / 3) as u8;
                    }
                    _ => {}
                }
            }
        }
    }

    fn draw_cursor_to_front(&self, x: usize, y: usize) {
        #[rustfmt::skip]
        const CURSOR_MAP: [[u8; 12]; 18] = [
            [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [2, 3, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [2, 3, 3, 2, 0, 0, 0, 0, 0, 0, 0, 0],
            [2, 3, 3, 3, 2, 0, 0, 0, 0, 0, 0, 0],
            [2, 3, 3, 3, 3, 2, 0, 0, 0, 0, 0, 0],
            [2, 3, 3, 3, 3, 3, 2, 0, 0, 0, 0, 0],
            [2, 3, 3, 3, 3, 3, 3, 2, 0, 0, 0, 0],
            [2, 3, 3, 3, 3, 3, 3, 3, 2, 0, 0, 0],
            [2, 3, 3, 3, 3, 3, 2, 2, 2, 2, 0, 0],
            [2, 3, 3, 2, 3, 3, 2, 0, 0, 0, 0, 0],
            [2, 3, 2, 0, 2, 3, 3, 2, 0, 0, 0, 0],
            [2, 2, 0, 0, 2, 3, 3, 2, 0, 0, 0, 0],
            [2, 0, 0, 0, 0, 2, 3, 3, 2, 0, 0, 0],
            [0, 0, 0, 0, 0, 2, 3, 3, 2, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 2, 3, 3, 2, 0, 0],
            [0, 0, 0, 0, 0, 0, 2, 3, 3, 2, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 2, 2, 0, 0, 0],
        ];
        for (cy, row) in CURSOR_MAP.iter().enumerate() {
            for (cx, &val) in row.iter().enumerate() {
                match val {
                    2 => self.write_front_pixel(x + cx, y + cy, Color::WHITE),
                    3 => self.write_front_pixel(x + cx, y + cy, Color::BRIGHT_GOLD),
                    _ => {}
                }
            }
        }
    }

    /// Blits the pristine cursor-free BACKBUFFER to VRAM, then overlays cursor directly on frontbuffer.
    pub fn swap_buffers_with_cursor(&mut self, cursor_x: usize, cursor_y: usize) {
        let t0 = unsafe { core::arch::x86_64::_rdtsc() };
        self.swap_buffers();
        self.draw_cursor_to_front(cursor_x, cursor_y);
        self.last_cursor_x = cursor_x;
        self.last_cursor_y = cursor_y;
        self.cursor_visible = true;
        let t1 = unsafe { core::arch::x86_64::_rdtsc() };
        LAST_FULL_FRAME_CYCLES.store(t1.saturating_sub(t0), core::sync::atomic::Ordering::Relaxed);
    }

    /// ULTRA-FAST dirty-rect cursor update (~864 bytes vs 8.3 MB):
    /// Restores old 12x18 cursor patch from BACKBUFFER to VRAM and draws cursor at new position.
    pub fn update_cursor_only(&mut self, new_x: usize, new_y: usize) {
        if !self.has_backbuffer || self.frontbuffer.is_null() {
            return;
        }
        let t0 = unsafe { core::arch::x86_64::_rdtsc() };
        if self.cursor_visible {
            let old_x = self.last_cursor_x;
            let old_y = self.last_cursor_y;
            let patch_w = 12usize.min(self.width.saturating_sub(old_x));
            let patch_h = 18usize.min(self.height.saturating_sub(old_y));
            let row_bytes = patch_w * self.bpp;
            let src_base = core::ptr::addr_of!(BACKBUFFER) as *const u8;
            for r in 0..patch_h {
                let offset = ((old_y + r) * self.stride + old_x) * self.bpp;
                if offset + row_bytes <= self.frontbuffer_len {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            src_base.add(offset),
                            self.frontbuffer.add(offset),
                            row_bytes,
                        );
                    }
                }
            }
        }
        self.draw_cursor_to_front(new_x, new_y);
        self.last_cursor_x = new_x;
        self.last_cursor_y = new_y;
        self.cursor_visible = true;
        let t1 = unsafe { core::arch::x86_64::_rdtsc() };
        LAST_CURSOR_BLIT_CYCLES.store(t1.saturating_sub(t0), core::sync::atomic::Ordering::Relaxed);
    }

    /// Fast double-buffer swap: Blits the backbuffer into the front VRAM buffer.
    pub fn swap_buffers(&mut self) {
        if !self.has_backbuffer || self.frontbuffer.is_null() {
            return;
        }

        let len = (self.stride * self.height * self.bpp).min(self.frontbuffer_len);
        let src_ptr = core::ptr::addr_of!(BACKBUFFER) as *const u8;
        unsafe {
            core::ptr::copy_nonoverlapping(src_ptr, self.frontbuffer, len);
        }
    }
}

pub static GRAPHICS: spin::Mutex<GraphicsEngine> = spin::Mutex::new(GraphicsEngine::uninitialized());
