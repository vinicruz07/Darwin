#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn to_u32(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub fn blend_over(self, bg: u32) -> u32 {
        if self.a == 255 {
            return self.to_u32();
        }
        if self.a == 0 {
            return bg;
        }

        let bg_r = ((bg >> 16) & 0xFF) as u32;
        let bg_g = ((bg >> 8) & 0xFF) as u32;
        let bg_b = (bg & 0xFF) as u32;

        let alpha = self.a as u32;
        let inv_alpha = 255 - alpha;

        let out_r = (self.r as u32 * alpha + bg_r * inv_alpha) / 255;
        let out_g = (self.g as u32 * alpha + bg_g * inv_alpha) / 255;
        let out_b = (self.b as u32 * alpha + bg_b * inv_alpha) / 255;

        (out_r << 16) | (out_g << 8) | out_b
    }
}

pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![0; width * height],
        }
    }

    pub fn clear(&mut self, color: Color) {
        let val = color.to_u32();
        self.buffer.fill(val);
    }

    #[inline(always)]
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height {
            let idx = (y as usize) * self.width + (x as usize);
            if color.a == 255 {
                self.buffer[idx] = color.to_u32();
            } else {
                self.buffer[idx] = color.blend_over(self.buffer[idx]);
            }
        }
    }

    pub fn draw_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, thickness: f32, color: Color) {
        if thickness <= 1.0 {
            self.draw_bresenham(x0 as i32, y0 as i32, x1 as i32, y1 as i32, color);
        } else {
            let dx = x1 - x0;
            let dy = y1 - y0;
            let len = (dx * dx + dy * dy).sqrt().max(1e-4);
            let half_t = thickness / 2.0;
            let nx = -dy / len * half_t;
            let ny = dx / len * half_t;

            let p0 = (x0 + nx, y0 + ny);
            let p1 = (x1 + nx, y1 + ny);
            let p2 = (x1 - nx, y1 - ny);
            let p3 = (x0 - nx, y0 - ny);

            self.draw_triangle(p0.0, p0.1, p1.0, p1.1, p2.0, p2.1, color);
            self.draw_triangle(p2.0, p2.1, p3.0, p3.1, p0.0, p0.1, color);
        }
    }

    fn draw_bresenham(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.set_pixel(x0, y0, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    pub fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        let x_start = (x.max(0.0) as usize).min(self.width);
        let y_start = (y.max(0.0) as usize).min(self.height);
        let x_end = ((x + w).max(0.0) as usize).min(self.width);
        let y_end = ((y + h).max(0.0) as usize).min(self.height);

        for py in y_start..y_end {
            let row_offset = py * self.width;
            for px in x_start..x_end {
                let idx = row_offset + px;
                if color.a == 255 {
                    self.buffer[idx] = color.to_u32();
                } else {
                    self.buffer[idx] = color.blend_over(self.buffer[idx]);
                }
            }
        }
    }

    pub fn draw_rect_lines(&mut self, x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
        self.draw_line(x, y, x + w, y, thickness, color);
        self.draw_line(x + w, y, x + w, y + h, thickness, color);
        self.draw_line(x + w, y + h, x, y + h, thickness, color);
        self.draw_line(x, y + h, x, y, thickness, color);
    }

    pub fn draw_circle(&mut self, cx: f32, cy: f32, r: f32, color: Color) {
        let x_start = ((cx - r).max(0.0) as i32).max(0);
        let y_start = ((cy - r).max(0.0) as i32).max(0);
        let x_end = ((cx + r + 1.0) as i32).min(self.width as i32 - 1);
        let y_end = ((cy + r + 1.0) as i32).min(self.height as i32 - 1);
        let r_sq = r * r;

        for py in y_start..=y_end {
            let dy = py as f32 - cy;
            for px in x_start..=x_end {
                let dx = px as f32 - cx;
                if dx * dx + dy * dy <= r_sq {
                    self.set_pixel(px, py, color);
                }
            }
        }
    }

    pub fn draw_circle_lines(&mut self, cx: f32, cy: f32, r: f32, color: Color) {
        let mut x = r as i32;
        let mut y = 0;
        let mut err = 0;

        let cx = cx as i32;
        let cy = cy as i32;

        while x >= y {
            self.set_pixel(cx + x, cy + y, color);
            self.set_pixel(cx + y, cy + x, color);
            self.set_pixel(cx - y, cy + x, color);
            self.set_pixel(cx - x, cy + y, color);
            self.set_pixel(cx - x, cy - y, color);
            self.set_pixel(cx - y, cy - x, color);
            self.set_pixel(cx + y, cy - x, color);
            self.set_pixel(cx + x, cy - y, color);

            y += 1;
            err += 1 + 2 * y;
            if 2 * (err - x) + 1 > 0 {
                x -= 1;
                err += 1 - 2 * x;
            }
        }
    }

    pub fn draw_triangle(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32, color: Color) {
        let min_x = x0.min(x1).min(x2).max(0.0) as i32;
        let max_x = x0.max(x1).max(x2).min(self.width as f32 - 1.0) as i32;
        let min_y = y0.min(y1).min(y2).max(0.0) as i32;
        let max_y = y0.max(y1).max(y2).min(self.height as f32 - 1.0) as i32;

        fn edge_function(ax: f32, ay: f32, bx: f32, by: f32, cx: f32, cy: f32) -> f32 {
            (cx - ax) * (by - ay) - (cy - ay) * (bx - ax)
        }

        let area = edge_function(x0, y0, x1, y1, x2, y2);
        if area.abs() < 1e-4 {
            return;
        }

        for py in min_y..=max_y {
            let cy = py as f32 + 0.5;
            for px in min_x..=max_x {
                let cx = px as f32 + 0.5;
                let w0 = edge_function(x1, y1, x2, y2, cx, cy);
                let w1 = edge_function(x2, y2, x0, y0, cx, cy);
                let w2 = edge_function(x0, y0, x1, y1, cx, cy);

                if (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0) {
                    self.set_pixel(px, py, color);
                }
            }
        }
    }

    /// Desenha texto ASCII usando uma fonte bitmap 8x8 embutida em software
    pub fn draw_text(&mut self, text: &str, x: f32, y: f32, scale: usize, color: Color) {
        let mut cur_x = x as i32;
        let cur_y = y as i32;
        let scale = scale.max(1);

        for ch in text.chars() {
            if ch == '\n' {
                cur_x = x as i32;
                continue;
            }

            let glyph = get_glyph(ch);
            for row in 0..8 {
                let row_bits = glyph[row];
                for col in 0..8 {
                    if (row_bits & (1 << (7 - col))) != 0 {
                        for sy in 0..scale {
                            for sx in 0..scale {
                                self.set_pixel(
                                    cur_x + (col * scale + sx) as i32,
                                    cur_y + (row * scale + sy) as i32,
                                    color,
                                );
                            }
                        }
                    }
                }
            }
            cur_x += (8 * scale) as i32;
        }
    }
}

// Fonte bitmap 8x8 compacta embutida para renderização independente de OS
fn get_glyph(c: char) -> [u8; 8] {
    let ascii = match c {
        'á' | 'à' | 'ã' | 'â' => 'a',
        'é' | 'ê' => 'e',
        'í' => 'i',
        'ó' | 'õ' | 'ô' => 'o',
        'ú' => 'u',
        'ç' => 'c',
        'Á' | 'À' | 'Ã' | 'Â' => 'A',
        'É' | 'Ê' => 'E',
        'Í' => 'I',
        'Ó' | 'Õ' | 'Ô' => 'O',
        'Ú' => 'U',
        'Ç' => 'C',
        _ => c,
    };

    match ascii {
        ' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        '!' => [0x18, 0x18, 0x18, 0x18, 0x18, 0x00, 0x18, 0x00],
        '"' => [0x66, 0x66, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00],
        '#' => [0x24, 0x24, 0x7E, 0x24, 0x7E, 0x24, 0x24, 0x00],
        '$' => [0x10, 0x7C, 0x90, 0x78, 0x14, 0x78, 0x10, 0x00],
        '%' => [0x62, 0x64, 0x08, 0x10, 0x20, 0x4C, 0x8C, 0x00],
        '&' => [0x30, 0x48, 0x30, 0x48, 0x8C, 0x92, 0x6C, 0x00],
        '\'' => [0x18, 0x18, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00],
        '(' => [0x0C, 0x18, 0x30, 0x30, 0x30, 0x18, 0x0C, 0x00],
        ')' => [0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x18, 0x30, 0x00],
        '*' => [0x00, 0x66, 0x3C, 0xFF, 0x3C, 0x66, 0x00, 0x00],
        '+' => [0x00, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x00, 0x00],
        ',' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x08],
        '-' => [0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00],
        '/' => [0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x00],
        '0' => [0x3C, 0x66, 0x6E, 0x76, 0x66, 0x66, 0x3C, 0x00],
        '1' => [0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x7E, 0x00],
        '2' => [0x3C, 0x66, 0x06, 0x1C, 0x30, 0x66, 0x7E, 0x00],
        '3' => [0x3C, 0x66, 0x06, 0x1C, 0x06, 0x66, 0x3C, 0x00],
        '4' => [0x0C, 0x1C, 0x34, 0x64, 0x7E, 0x04, 0x04, 0x00],
        '5' => [0x7E, 0x60, 0x7C, 0x06, 0x06, 0x66, 0x3C, 0x00],
        '6' => [0x3C, 0x66, 0x60, 0x7C, 0x66, 0x66, 0x3C, 0x00],
        '7' => [0x7E, 0x66, 0x0C, 0x18, 0x18, 0x18, 0x18, 0x00],
        '8' => [0x3C, 0x66, 0x66, 0x3C, 0x66, 0x66, 0x3C, 0x00],
        '9' => [0x3C, 0x66, 0x66, 0x3E, 0x06, 0x66, 0x3C, 0x00],
        ':' => [0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x00],
        ';' => [0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x08],
        '<' => [0x06, 0x1C, 0x70, 0x1C, 0x06, 0x00, 0x00, 0x00],
        '=' => [0x00, 0x00, 0x7E, 0x00, 0x7E, 0x00, 0x00, 0x00],
        '>' => [0x60, 0x38, 0x0E, 0x38, 0x60, 0x00, 0x00, 0x00],
        '?' => [0x3C, 0x66, 0x0C, 0x18, 0x18, 0x00, 0x18, 0x00],
        '@' => [0x3C, 0x42, 0x9D, 0xA5, 0x9D, 0x40, 0x3C, 0x00],
        'A' => [0x18, 0x3C, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x00],
        'B' => [0x7C, 0x66, 0x66, 0x7C, 0x66, 0x66, 0x7C, 0x00],
        'C' => [0x3C, 0x66, 0x60, 0x60, 0x60, 0x66, 0x3C, 0x00],
        'D' => [0x78, 0x6C, 0x66, 0x66, 0x66, 0x6C, 0x78, 0x00],
        'E' => [0x7E, 0x60, 0x60, 0x7C, 0x60, 0x60, 0x7E, 0x00],
        'F' => [0x7E, 0x60, 0x60, 0x7C, 0x60, 0x60, 0x60, 0x00],
        'G' => [0x3C, 0x66, 0x60, 0x6E, 0x66, 0x66, 0x3C, 0x00],
        'H' => [0x66, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x00],
        'I' => [0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x7E, 0x00],
        'J' => [0x1E, 0x0C, 0x0C, 0x0C, 0x6C, 0x6C, 0x38, 0x00],
        'K' => [0x66, 0x6C, 0x78, 0x70, 0x78, 0x6C, 0x66, 0x00],
        'L' => [0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x7E, 0x00],
        'M' => [0x63, 0x77, 0x7F, 0x6B, 0x63, 0x63, 0x63, 0x00],
        'N' => [0x66, 0x76, 0x7E, 0x7E, 0x6E, 0x66, 0x66, 0x00],
        'O' => [0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00],
        'P' => [0x7C, 0x66, 0x66, 0x7C, 0x60, 0x60, 0x60, 0x00],
        'Q' => [0x3C, 0x66, 0x66, 0x66, 0x6E, 0x3C, 0x0E, 0x00],
        'R' => [0x7C, 0x66, 0x66, 0x7C, 0x78, 0x6C, 0x66, 0x00],
        'S' => [0x3C, 0x66, 0x60, 0x3C, 0x06, 0x66, 0x3C, 0x00],
        'T' => [0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00],
        'U' => [0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00],
        'V' => [0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00],
        'W' => [0x63, 0x63, 0x63, 0x6B, 0x7F, 0x77, 0x63, 0x00],
        'X' => [0x66, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0x66, 0x00],
        'Y' => [0x66, 0x66, 0x66, 0x3C, 0x18, 0x18, 0x18, 0x00],
        'Z' => [0x7E, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x7E, 0x00],
        '[' => [0x3C, 0x30, 0x30, 0x30, 0x30, 0x30, 0x3C, 0x00],
        '\\' => [0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x00],
        ']' => [0x3C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x3C, 0x00],
        '^' => [0x18, 0x3C, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00],
        '_' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7E, 0x00],
        'a' => [0x00, 0x00, 0x3C, 0x06, 0x3E, 0x66, 0x3E, 0x00],
        'b' => [0x60, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x7C, 0x00],
        'c' => [0x00, 0x00, 0x3C, 0x66, 0x60, 0x66, 0x3C, 0x00],
        'd' => [0x06, 0x06, 0x3E, 0x66, 0x66, 0x66, 0x3E, 0x00],
        'e' => [0x00, 0x00, 0x3C, 0x66, 0x7E, 0x60, 0x3C, 0x00],
        'f' => [0x1C, 0x30, 0x7C, 0x30, 0x30, 0x30, 0x30, 0x00],
        'g' => [0x00, 0x00, 0x3E, 0x66, 0x66, 0x3E, 0x06, 0x3C],
        'h' => [0x60, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x66, 0x00],
        'i' => [0x18, 0x00, 0x38, 0x18, 0x18, 0x18, 0x3C, 0x00],
        'j' => [0x0C, 0x00, 0x1C, 0x0C, 0x0C, 0x0C, 0x4C, 0x38],
        'k' => [0x60, 0x60, 0x66, 0x6C, 0x78, 0x6C, 0x66, 0x00],
        'l' => [0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00],
        'm' => [0x00, 0x00, 0x66, 0x7F, 0x7F, 0x6B, 0x63, 0x00],
        'n' => [0x00, 0x00, 0x7C, 0x66, 0x66, 0x66, 0x66, 0x00],
        'o' => [0x00, 0x00, 0x3C, 0x66, 0x66, 0x66, 0x3C, 0x00],
        'p' => [0x00, 0x00, 0x7C, 0x66, 0x66, 0x7C, 0x60, 0x60],
        'q' => [0x00, 0x00, 0x3E, 0x66, 0x66, 0x3E, 0x06, 0x06],
        'r' => [0x00, 0x00, 0x7C, 0x66, 0x60, 0x60, 0x60, 0x00],
        's' => [0x00, 0x00, 0x3E, 0x60, 0x3C, 0x06, 0x7C, 0x00],
        't' => [0x18, 0x18, 0x7E, 0x18, 0x18, 0x18, 0x0E, 0x00],
        'u' => [0x00, 0x00, 0x66, 0x66, 0x66, 0x66, 0x3E, 0x00],
        'v' => [0x00, 0x00, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00],
        'w' => [0x00, 0x00, 0x63, 0x6B, 0x7F, 0x3E, 0x36, 0x00],
        'x' => [0x00, 0x00, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0x00],
        'y' => [0x00, 0x00, 0x66, 0x66, 0x66, 0x3E, 0x06, 0x3C],
        'z' => [0x00, 0x00, 0x7E, 0x0C, 0x18, 0x30, 0x7E, 0x00],
        '{' => [0x0E, 0x18, 0x18, 0x70, 0x18, 0x18, 0x0E, 0x00],
        '|' => [0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00],
        '}' => [0x70, 0x18, 0x18, 0x0E, 0x18, 0x18, 0x70, 0x00],
        '~' => [0x76, 0xDC, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        _ => [0xFF, 0x81, 0x81, 0x81, 0x81, 0x81, 0xFF, 0x00], // Caractere desconhecido
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_primitives_and_bounds() {
        let mut canvas = Canvas::new(100, 100);
        canvas.clear(Color::rgb(0, 0, 0));

        // Testar se operações fora dos limites não causam panic
        canvas.set_pixel(-10, -10, Color::rgb(255, 255, 255));
        canvas.set_pixel(150, 150, Color::rgb(255, 255, 255));
        canvas.draw_line(-50.0, -50.0, 200.0, 200.0, 2.0, Color::rgb(255, 0, 0));
        canvas.draw_rect(-20.0, -20.0, 50.0, 50.0, Color::rgb(0, 255, 0));
        canvas.draw_circle(50.0, 50.0, 20.0, Color::rgba(0, 0, 255, 128));
        canvas.draw_text("TESTE 123!", 10.0, 10.0, 1, Color::rgb(255, 255, 255));

        assert_eq!(canvas.buffer.len(), 100 * 100);
    }
}

