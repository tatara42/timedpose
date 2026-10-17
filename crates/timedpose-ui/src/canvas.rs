use egui::{pos2, vec2, Color32, ColorImage, Painter, Rect, Stroke, TextureHandle, Vec2};
use timedpose_infra::DecodedImage;

/// Filter configuration applied to the reference image.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasFilterSettings {
    pub flip_horizontal: bool,
    pub invert_upside_down: bool,
    pub grayscale: bool,
    pub threshold: bool,
    pub threshold_cutoff: f32, // 0.0 to 1.0
    pub squint_blur: bool,
    pub squint_radius: usize, // e.g. 1 to 8
    pub grid_overlay: bool,
}

impl Default for CanvasFilterSettings {
    fn default() -> Self {
        Self {
            flip_horizontal: false,
            invert_upside_down: false,
            grayscale: false,
            threshold: false,
            threshold_cutoff: 0.5,
            squint_blur: false,
            squint_radius: 4,
            grid_overlay: false,
        }
    }
}

/// Applies perceptual filters (Grayscale, Thresholding, Squint Blur) to raw image pixels,
/// converting them into an `egui::ColorImage`.
pub fn process_image_filters(raw: &DecodedImage, filters: &CanvasFilterSettings) -> ColorImage {
    let width = raw.width as usize;
    let height = raw.height as usize;

    // Fast path: if no pixel filters are active, avoid cloning the raw buffer
    if !filters.squint_blur && !filters.threshold && !filters.grayscale {
        return ColorImage::from_rgba_unmultiplied([width, height], &raw.rgba);
    }

    let mut pixels = raw.rgba.clone();

    // 1. Squint Mode (Blur filter to eliminate micro-details)
    if filters.squint_blur && filters.squint_radius > 0 {
        pixels = apply_box_blur(&pixels, width, height, filters.squint_radius);
    }

    // 2. Grayscale & Thresholding
    if filters.threshold {
        let cutoff_u8 = (filters.threshold_cutoff * 255.0).clamp(0.0, 255.0) as u8;
        for chunk in pixels.chunks_exact_mut(4) {
            let r = chunk[0] as u32;
            let g = chunk[1] as u32;
            let b = chunk[2] as u32;
            // Standard perceptual luminance
            let lum = ((r * 77 + g * 150 + b * 29) >> 8) as u8;
            let val = if lum >= cutoff_u8 { 255 } else { 0 };
            chunk[0] = val;
            chunk[1] = val;
            chunk[2] = val;
        }
    } else if filters.grayscale {
        for chunk in pixels.chunks_exact_mut(4) {
            let r = chunk[0] as u32;
            let g = chunk[1] as u32;
            let b = chunk[2] as u32;
            let lum = ((r * 77 + g * 150 + b * 29) >> 8) as u8;
            chunk[0] = lum;
            chunk[1] = lum;
            chunk[2] = lum;
        }
    }

    ColorImage::from_rgba_unmultiplied([width, height], &pixels)
}

/// Fast separable 2D box blur simulating the squinting effect.
fn apply_box_blur(src: &[u8], width: usize, height: usize, radius: usize) -> Vec<u8> {
    if radius == 0 || width == 0 || height == 0 {
        return src.to_vec();
    }

    let mut temp = vec![0u8; src.len()];
    let mut out = vec![0u8; src.len()];
    let r = radius as isize;

    // Horizontal pass
    for y in 0..height {
        for x in 0..width {
            let mut sum_r = 0;
            let mut sum_g = 0;
            let mut sum_b = 0;
            let mut count = 0;

            for dx in -r..=r {
                let nx = x as isize + dx;
                if nx >= 0 && nx < width as isize {
                    let idx = (y * width + nx as usize) * 4;
                    sum_r += src[idx] as u32;
                    sum_g += src[idx + 1] as u32;
                    sum_b += src[idx + 2] as u32;
                    count += 1;
                }
            }

            let out_idx = (y * width + x) * 4;
            temp[out_idx] = (sum_r / count) as u8;
            temp[out_idx + 1] = (sum_g / count) as u8;
            temp[out_idx + 2] = (sum_b / count) as u8;
            temp[out_idx + 3] = src[out_idx + 3];
        }
    }

    // Vertical pass
    for x in 0..width {
        for y in 0..height {
            let mut sum_r = 0;
            let mut sum_g = 0;
            let mut sum_b = 0;
            let mut count = 0;

            for dy in -r..=r {
                let ny = y as isize + dy;
                if ny >= 0 && ny < height as isize {
                    let idx = (ny as usize * width + x) * 4;
                    sum_r += temp[idx] as u32;
                    sum_g += temp[idx + 1] as u32;
                    sum_b += temp[idx + 2] as u32;
                    count += 1;
                }
            }

            let out_idx = (y * width + x) * 4;
            out[out_idx] = (sum_r / count) as u8;
            out[out_idx + 1] = (sum_g / count) as u8;
            out[out_idx + 2] = (sum_b / count) as u8;
            out[out_idx + 3] = temp[out_idx + 3];
        }
    }

    out
}

/// Renders the reference image centered in the available rectangle,
/// handling pan, zoom, UV mirroring, and grid overlays.
pub fn render_canvas(
    ui: &mut egui::Ui,
    texture: &TextureHandle,
    filters: &CanvasFilterSettings,
    pan: &mut Vec2,
    zoom: &mut f32,
) -> Rect {
    let available_rect = ui.available_rect_before_wrap();
    // Use drag only (not click) so canvas doesn't contest clicks with overlay buttons
    let response = ui.allocate_rect(available_rect, egui::Sense::drag());

    let is_over_toolbar = ui.input(|i| {
        if let Some(pos) = i.pointer.hover_pos() {
            pos.y > available_rect.bottom() - 65.0
        } else {
            false
        }
    });

    // Pan with middle mouse drag, right mouse drag, or left drag when zoomed in (and not over toolbar)
    if response.dragged() && !is_over_toolbar {
        if response.dragged_by(egui::PointerButton::Middle)
            || response.dragged_by(egui::PointerButton::Secondary)
            || (*zoom > 1.05 && response.dragged_by(egui::PointerButton::Primary))
        {
            *pan += response.drag_delta();
        }
    }

    // Zoom with scroll wheel (only if not over toolbar)
    if !is_over_toolbar {
        let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll_delta != 0.0 {
            let zoom_factor = if scroll_delta > 0.0 { 1.1 } else { 0.9 };
            *zoom = (*zoom * zoom_factor).clamp(0.2, 8.0);
        }
    }

    // Calculate aspect-ratio fit
    let tex_size = texture.size_vec2();
    let aspect = tex_size.x / tex_size.y.max(1.0);

    let (mut fit_w, mut fit_h) = if available_rect.width() / available_rect.height().max(1.0) > aspect {
        // Limited by height
        let h = available_rect.height();
        (h * aspect, h)
    } else {
        // Limited by width
        let w = available_rect.width();
        (w, w / aspect)
    };

    fit_w *= *zoom;
    fit_h *= *zoom;

    let center = available_rect.center() + *pan;
    let image_rect = Rect::from_center_size(center, vec2(fit_w, fit_h));

    // UV calculations for instant 0ms horizontal mirror and upside-down transforms
    let u_min = if filters.flip_horizontal { 1.0 } else { 0.0 };
    let u_max = if filters.flip_horizontal { 0.0 } else { 1.0 };
    let v_min = if filters.invert_upside_down { 1.0 } else { 0.0 };
    let v_max = if filters.invert_upside_down { 0.0 } else { 1.0 };
    let uv = Rect::from_min_max(pos2(u_min, v_min), pos2(u_max, v_max));

    let painter = ui.painter_at(available_rect);

    // Draw the image
    painter.image(texture.id(), image_rect, uv, Color32::WHITE);

    // Draw Rule of Thirds & Center Grid if enabled
    if filters.grid_overlay {
        draw_grid(&painter, image_rect);
    }

    image_rect
}

fn draw_grid(painter: &Painter, rect: Rect) {
    let stroke = Stroke::new(1.0_f32, Color32::from_white_alpha(70));
    let center_stroke = Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(255, 200, 50, 100));

    // Rule of Thirds - Vertical lines
    let x_one_third = rect.left() + rect.width() / 3.0;
    let x_two_thirds = rect.left() + 2.0 * rect.width() / 3.0;
    painter.line_segment([pos2(x_one_third, rect.top()), pos2(x_one_third, rect.bottom())], stroke);
    painter.line_segment([pos2(x_two_thirds, rect.top()), pos2(x_two_thirds, rect.bottom())], stroke);

    // Rule of Thirds - Horizontal lines
    let y_one_third = rect.top() + rect.height() / 3.0;
    let y_two_thirds = rect.top() + 2.0 * rect.height() / 3.0;
    painter.line_segment([pos2(rect.left(), y_one_third), pos2(rect.right(), y_one_third)], stroke);
    painter.line_segment([pos2(rect.left(), y_two_thirds), pos2(rect.right(), y_two_thirds)], stroke);

    // Center Crosshair
    let center = rect.center();
    painter.line_segment([pos2(center.x, center.y - 15.0), pos2(center.x, center.y + 15.0)], center_stroke);
    painter.line_segment([pos2(center.x - 15.0, center.y), pos2(center.x + 15.0, center.y)], center_stroke);
}
