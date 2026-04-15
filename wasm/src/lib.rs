use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn apply_levels_rgba(data: &[u8], black_point: u8, white_point: u8) -> Vec<u8> {
    if data.len() % 4 != 0 || black_point >= white_point {
        return data.to_vec();
    }

    let mut output = data.to_vec();
    let input_range = (white_point as f32 - black_point as f32).max(1.0);

    for pixel in output.chunks_exact_mut(4) {
        for channel in pixel.iter_mut().take(3) {
            let value = *channel as f32;
            let normalized = ((value - black_point as f32) / input_range).clamp(0.0, 1.0);
            *channel = (normalized * 255.0).round() as u8;
        }
    }

    output
}

#[wasm_bindgen]
pub fn adjust_hue_saturation_rgba(data: &[u8], hue_shift_degrees: f32, saturation_scale: f32) -> Vec<u8> {
    if data.len() % 4 != 0 {
        return data.to_vec();
    }

    let mut output = data.to_vec();

    for pixel in output.chunks_exact_mut(4) {
        let (hue, saturation, value) = rgb_to_hsv(pixel[0], pixel[1], pixel[2]);
        let next_hue = wrap_hue(hue + hue_shift_degrees);
        let next_saturation = (saturation * saturation_scale).clamp(0.0, 1.0);
        let (red, green, blue) = hsv_to_rgb(next_hue, next_saturation, value);

        pixel[0] = red;
        pixel[1] = green;
        pixel[2] = blue;
    }

    output
}

#[wasm_bindgen]
pub fn gaussian_blur_rgba(data: &[u8], width: u32, height: u32, radius: u32) -> Vec<u8> {
    if data.len() % 4 != 0 || width == 0 || height == 0 || radius == 0 {
        return data.to_vec();
    }

    let expected_len = width as usize * height as usize * 4;
    if data.len() != expected_len {
        return data.to_vec();
    }

    let mut horizontal = vec![0_u8; data.len()];
    let mut output = vec![0_u8; data.len()];

    // Separable blur pass 1: horizontal.
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            for channel in 0..4 {
                let mut total = 0_u32;
                let mut count = 0_u32;

                for sample_x in (x - radius as i32)..=(x + radius as i32) {
                    let clamped_x = sample_x.clamp(0, width as i32 - 1);
                    let idx = index(width, clamped_x as u32, y as u32, channel);
                    total += data[idx] as u32;
                    count += 1;
                }

                let destination = index(width, x as u32, y as u32, channel);
                horizontal[destination] = (total / count) as u8;
            }
        }
    }

    // Separable blur pass 2: vertical.
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            for channel in 0..4 {
                let mut total = 0_u32;
                let mut count = 0_u32;

                for sample_y in (y - radius as i32)..=(y + radius as i32) {
                    let clamped_y = sample_y.clamp(0, height as i32 - 1);
                    let idx = index(width, x as u32, clamped_y as u32, channel);
                    total += horizontal[idx] as u32;
                    count += 1;
                }

                let destination = index(width, x as u32, y as u32, channel);
                output[destination] = (total / count) as u8;
            }
        }
    }

    output
}

fn index(width: u32, x: u32, y: u32, channel: usize) -> usize {
    ((y * width + x) as usize * 4) + channel
}

fn wrap_hue(hue: f32) -> f32 {
    let mut value = hue % 360.0;
    if value < 0.0 {
        value += 360.0;
    }
    value
}

fn rgb_to_hsv(red: u8, green: u8, blue: u8) -> (f32, f32, f32) {
    let r = red as f32 / 255.0;
    let g = green as f32 / 255.0;
    let b = blue as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let hue = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let saturation = if max == 0.0 { 0.0 } else { delta / max };
    (wrap_hue(hue), saturation, max)
}

fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> (u8, u8, u8) {
    let chroma = value * saturation;
    let hue_segment = hue / 60.0;
    let second = chroma * (1.0 - ((hue_segment % 2.0) - 1.0).abs());
    let match_value = value - chroma;

    let (r1, g1, b1) = if (0.0..1.0).contains(&hue_segment) {
        (chroma, second, 0.0)
    } else if (1.0..2.0).contains(&hue_segment) {
        (second, chroma, 0.0)
    } else if (2.0..3.0).contains(&hue_segment) {
        (0.0, chroma, second)
    } else if (3.0..4.0).contains(&hue_segment) {
        (0.0, second, chroma)
    } else if (4.0..5.0).contains(&hue_segment) {
        (second, 0.0, chroma)
    } else {
        (chroma, 0.0, second)
    };

    let red = ((r1 + match_value) * 255.0).round().clamp(0.0, 255.0) as u8;
    let green = ((g1 + match_value) * 255.0).round().clamp(0.0, 255.0) as u8;
    let blue = ((b1 + match_value) * 255.0).round().clamp(0.0, 255.0) as u8;

    (red, green, blue)
}

#[cfg(test)]
mod tests {
    use super::{adjust_hue_saturation_rgba, apply_levels_rgba, gaussian_blur_rgba};

    #[test]
    fn apply_levels_should_scale_channels() {
        let pixels = vec![20, 40, 60, 255, 200, 220, 240, 255];
        let output = apply_levels_rgba(&pixels, 20, 220);

        assert_eq!(output[0], 0);
        assert_eq!(output[4], 230);
        assert_eq!(output[3], 255);
    }

    #[test]
    fn hue_adjustment_should_change_rgb_channels() {
        let pixels = vec![255, 0, 0, 255];
        let output = adjust_hue_saturation_rgba(&pixels, 120.0, 1.0);

        assert_ne!(output[0], 255);
        assert_eq!(output[3], 255);
    }

    #[test]
    fn blur_should_preserve_buffer_size() {
        let pixels = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];

        let output = gaussian_blur_rgba(&pixels, 2, 2, 1);
        assert_eq!(output.len(), pixels.len());
    }
}
