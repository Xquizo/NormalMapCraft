#![no_std]

use core::arch::wasm32;
use core::slice;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

static MANIFEST: &[u8] = br#"{
  "id":"org.photocraft.community.normal-map",
  "name":"Normal Map Pro (Multi-Scale)",
  "version":"1.4.0",
  "kind":"filter",
  "author":"Custom",
  "description":"Multi-scale Normal Map generator with detail scale, bevel depth, directional orientation, pre-blur, contrast and alpha fill.",
  "params":{
    "detail_scale":{"type":"choice","options":["Bajo (1.5)","Medio (3.5)","Alto (6.0)","Muy Alto (10.0)","Extremo (16.0)"],"default":"Medio (3.5)"},
    "bevel":{"type":"choice","options":["Fino (1 px)","Medio (3 px)","Ancho (6 px)","Volumetrico (10 px)"],"default":"Medio (3 px)"},
    "direction":{"type":"choice","options":["Seleccionar...","OpenGL (+Y / Unity)","Sur (3ds Max / DirectX)","Este","Oeste"],"default":"Seleccionar..."},
    "blur":{"type":"choice","options":["0 (Ninguno)","1 px (Sutil)","2 px (Suave)"],"default":"0 (Ninguno)"},
    "contrast":{"type":"choice","options":["Lineal (1.0)","Medio (1.3)","Fuerte (1.7)"],"default":"Lineal (1.0)"},
    "transparency":{"type":"choice","options":["Rellenar neutro (#8080FF)","Conservar alfa"],"default":"Rellenar neutro (#8080FF)"}
  },
  "overlap":12,
  "area":"canvas"
}"#;

#[cfg(target_arch = "wasm32")]
static mut HEAP_NEXT: usize = 65536;

#[no_mangle]
pub extern "C" fn pc_abi_version() -> i32 {
    1
}

#[no_mangle]
pub extern "C" fn pc_manifest() -> i64 {
    ((MANIFEST.len() as i64) << 32) | (MANIFEST.as_ptr() as u32 as i64)
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn pc_alloc(size: i32) -> i32 {
    if size <= 0 {
        return 0;
    }
    let size = size as usize;
    let start = unsafe { (HEAP_NEXT + 7) & !7usize };
    let end = match start.checked_add(size) {
        Some(value) => value,
        None => return 0,
    };
    let pages_needed = end.saturating_add(65535) / 65536;
    let pages_now = wasm32::memory_size(0);
    if pages_needed > pages_now {
        let grown = wasm32::memory_grow(0, pages_needed - pages_now);
        if grown == usize::MAX {
            return 0;
        }
    }
    unsafe {
        HEAP_NEXT = end;
    }
    start as i32
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
pub extern "C" fn pc_alloc(_size: i32) -> i32 {
    0
}

#[inline]
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

fn parse_options(params: &[u8]) -> (f32, isize, u8, i32, f32, bool) {
    let strength = if contains(params, b"Bajo") {
        1.5
    } else if contains(params, b"Muy Alto") {
        10.0
    } else if contains(params, b"Alto") {
        6.0
    } else if contains(params, b"Extremo") {
        16.0
    } else {
        3.5
    };

    let bevel_radius = if contains(params, b"Volum") {
        10
    } else if contains(params, b"Ancho") {
        6
    } else if contains(params, b"Fino") {
        1
    } else {
        3 // Medio (3 px)
    };

    let dir = if contains(params, b"OpenGL") {
        0
    } else if contains(params, b"Sur") {
        1
    } else if contains(params, b"Este") {
        2
    } else if contains(params, b"Oeste") {
        3
    } else {
        255
    };

    let blur_radius = if contains(params, b"2 px") {
        2
    } else if contains(params, b"1 px") {
        1
    } else {
        0
    };

    let contrast = if contains(params, b"Fuerte") {
        1.7
    } else if contains(params, b"1.3") {
        1.3
    } else {
        1.0
    };

    let fill_neutral = !contains(params, b"Conservar");

    (strength, bevel_radius, dir, blur_radius, contrast, fill_neutral)
}

#[inline(always)]
fn sqrt_f32(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    let mut g = f32::from_bits(0x1fbd1df5 + (x.to_bits() >> 1));
    g = 0.5 * (g + x / g);
    g = 0.5 * (g + x / g);
    0.5 * (g + x / g)
}

#[inline(always)]
fn apply_contrast(lum: f32, gamma: f32) -> f32 {
    if gamma <= 1.01 {
        return lum;
    }
    let l = lum.clamp(0.0, 1.0);
    let centered = l - 0.5;
    let factor = if gamma > 1.5 { 2.2 } else { 1.5 };
    (0.5 + centered * factor).clamp(0.0, 1.0)
}

// Clamp al borde del tile (sin wrap-around) para evitar costuras entre cuadrantes
#[inline(always)]
fn sample_ch(
    pixels: &[f32],
    x: isize,
    y: isize,
    width: isize,
    height: isize,
    w: usize,
    ch: usize,
    offset: usize,
) -> f32 {
    let cx = x.clamp(0, width - 1) as usize;
    let cy = y.clamp(0, height - 1) as usize;
    pixels[(cy * w + cx) * ch + offset]
}

#[inline(always)]
fn sobel_at_step(
    pixels: &[f32],
    x: isize,
    y: isize,
    step: isize,
    wi: isize,
    hi: isize,
    w: usize,
    ch: usize,
    lum_offset: usize,
) -> (f32, f32) {
    let tl = sample_ch(pixels, x - step, y - step, wi, hi, w, ch, lum_offset);
    let t  = sample_ch(pixels, x,        y - step, wi, hi, w, ch, lum_offset);
    let tr = sample_ch(pixels, x + step, y - step, wi, hi, w, ch, lum_offset);
    let l  = sample_ch(pixels, x - step, y,        wi, hi, w, ch, lum_offset);
    let r  = sample_ch(pixels, x + step, y,        wi, hi, w, ch, lum_offset);
    let bl = sample_ch(pixels, x - step, y + step, wi, hi, w, ch, lum_offset);
    let b  = sample_ch(pixels, x,        y + step, wi, hi, w, ch, lum_offset);
    let br = sample_ch(pixels, x + step, y + step, wi, hi, w, ch, lum_offset);

    let dx = (tr + 2.0 * r + br) - (tl + 2.0 * l + bl);
    let dy = (bl + 2.0 * b + br) - (tl + 2.0 * t + tr);
    (dx, dy)
}

#[no_mangle]
pub unsafe extern "C" fn pc_filter(
    buf: i32,
    buf_len: i32,
    width: i32,
    height: i32,
    channels: i32,
    _format: i32,
    params_ptr: i32,
    params_len: i32,
) -> i32 {
    if buf <= 0 || width <= 0 || height <= 0 || channels < 3 || params_ptr <= 0 || params_len < 0 {
        return 1;
    }

    let w = width as usize;
    let h = height as usize;
    let ch = channels as usize;
    let count = w * h * ch;

    if count * 4 != (buf_len as usize) {
        return 3;
    }

    let params = slice::from_raw_parts(params_ptr as *const u8, params_len as usize);
    let (strength, bevel_radius, dir, blur_radius, contrast, fill_neutral) = parse_options(params);

    let pixels = slice::from_raw_parts_mut(buf as *mut f32, count);
    let num_pixels = w * h;

    if dir == 255 {
        for i in 0..num_pixels {
            let p = i * ch;
            pixels[p]     = 0.5019608;
            pixels[p + 1] = 0.5019608;
            pixels[p + 2] = 1.0;
            if ch >= 4 && fill_neutral {
                pixels[p + 3] = 1.0;
            }
        }
        return 0;
    }

    let wi = width as isize;
    let hi = height as isize;

    // 1. Extraer luminancia en canal R
    for i in 0..num_pixels {
        let p = i * ch;
        let is_transparent = ch >= 4 && pixels[p + 3] <= 0.01;
        if is_transparent {
            pixels[p] = 0.5;
        } else {
            let lum = 0.2126 * pixels[p] + 0.7152 * pixels[p + 1] + 0.0722 * pixels[p + 2];
            pixels[p] = apply_contrast(lum, contrast);
        }
    }

    // 2. Desenfoque previo opcional
    let lum_offset = if blur_radius > 0 {
        for y in 0..hi {
            for x in 0..wi {
                let mut sum = 0.0f32;
                let mut samples = 0.0f32;
                for dy in -blur_radius..=blur_radius {
                    for dx in -blur_radius..=blur_radius {
                        sum += sample_ch(pixels, x + dx as isize, y + dy as isize, wi, hi, w, ch, 0);
                        samples += 1.0;
                    }
                }
                let p = (y as usize * w + x as usize) * ch;
                pixels[p + 1] = sum / samples;
            }
        }
        1
    } else {
        0
    };

    // 3. Sobel Multi-Escala (combina detalle fino de 1px + volumen ancho según bevel_radius)
    for y in 0..hi {
        for x in 0..wi {
            let (dx1, dy1) = sobel_at_step(pixels, x, y, 1, wi, hi, w, ch, lum_offset);

            let (dx, dy) = if bevel_radius <= 1 {
                (dx1, dy1)
            } else {
                let mid = (bevel_radius / 2).max(2);
                let (dxm, dym) = sobel_at_step(pixels, x, y, mid, wi, hi, w, ch, lum_offset);
                let (dxw, dyw) = sobel_at_step(pixels, x, y, bevel_radius, wi, hi, w, ch, lum_offset);
                // Mezcla piramidal: 45% fino + 35% medio + 20% ancho
                (
                    dx1 * 0.45 + dxm * 0.35 + dxw * 0.20,
                    dy1 * 0.45 + dym * 0.35 + dyw * 0.20,
                )
            };

            let (gx, gy) = match dir {
                1 => (-dx,  dy),
                2 => (-dy, -dx),
                3 => ( dy,  dx),
                _ => (-dx, -dy),
            };

            let mut nx = gx * strength;
            let mut ny = gy * strength;

            let len = sqrt_f32(nx * nx + ny * ny + 1.0);
            nx /= len;
            ny /= len;

            let p = (y as usize * w + x as usize) * ch;
            let out_r = (nx * 0.5 + 0.5).clamp(0.0, 1.0);
            let out_g = (ny * 0.5 + 0.5).clamp(0.0, 1.0);

            if lum_offset == 0 {
                pixels[p + 1] = out_r;
                pixels[p + 2] = out_g;
            } else {
                pixels[p]     = out_r;
                pixels[p + 2] = out_g;
            }
        }
    }

    // 4. Reconstruir Z y aplicar política de transparencia
    for i in 0..num_pixels {
        let p = i * ch;
        let is_transparent = ch >= 4 && pixels[p + 3] <= 0.01;

        if is_transparent && fill_neutral {
            pixels[p]     = 0.5019608;
            pixels[p + 1] = 0.5019608;
            pixels[p + 2] = 1.0;
            pixels[p + 3] = 1.0;
        } else {
            let (out_r, out_g) = if lum_offset == 0 {
                (pixels[p + 1], pixels[p + 2])
            } else {
                (pixels[p], pixels[p + 2])
            };

            let nx = out_r * 2.0 - 1.0;
            let ny = out_g * 2.0 - 1.0;
            let nz = sqrt_f32((1.0 - nx * nx - ny * ny).max(0.0));

            pixels[p]     = out_r;
            pixels[p + 1] = out_g;
            pixels[p + 2] = (nz * 0.5 + 0.5).clamp(0.0, 1.0);
        }
    }

    0
}