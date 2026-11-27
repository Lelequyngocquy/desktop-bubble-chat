use std::ffi::c_void;

use crate::operating_systems::windows::bubble::window::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, SelectObject, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, RECT, SIZE,
};
use crate::presentation::bubble::dock::DockTooltipPillRect;

const TRANSPARENT_BK: i32 = 1;
const FW_MEDIUM: i32 = 500;
const DEFAULT_CHARSET: u32 = 1;
const OUT_DEFAULT_PRECIS: u32 = 0;
const CLIP_DEFAULT_PRECIS: u32 = 0;
const ANTIALIASED_QUALITY: u32 = 4;
const FF_SWISS: u32 = 2 << 4;
const DT_CENTER: u32 = 0x0001;
const DT_VCENTER: u32 = 0x0004;
const DT_SINGLELINE: u32 = 0x0020;
const DT_NOPREFIX: u32 = 0x0800;

#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateFontW(
        c_height: i32,
        c_width: i32,
        c_escapement: i32,
        c_orientation: i32,
        c_weight: i32,
        b_italic: u32,
        b_underline: u32,
        b_strike_out: u32,
        i_char_set: u32,
        i_out_precision: u32,
        i_clip_precision: u32,
        i_quality: u32,
        i_pitch_and_family: u32,
        psz_face_name: *const u16,
    ) -> *mut c_void;
    fn SetBkMode(hdc: *mut c_void, mode: i32) -> i32;
    fn SetTextColor(hdc: *mut c_void, color: u32) -> u32;
    fn GetTextExtentPoint32W(hdc: *mut c_void, lp_string: *const u16, c: i32, psizl: *mut SIZE) -> i32;
}

#[link(name = "user32")]
unsafe extern "system" {
    fn DrawTextW(hdc: *mut c_void, lpch_text: *const u16, cch_text: i32, lprc: *mut RECT, format: u32) -> i32;
}

/// Creates an anti-aliased Segoe UI font scaled cleanly for the monitor DPI
unsafe fn create_tooltip_font(scale_factor: f64) -> *mut c_void {
    let scale = if scale_factor.is_finite() && scale_factor > 0.25 { scale_factor } else { 1.0 };
    let font_height = -((12.0 * scale).round() as i32).max(10);
    let face: Vec<u16> = "Segoe UI\0".encode_utf16().collect();
    unsafe {
        CreateFontW(
            font_height,
            0,
            0,
            0,
            FW_MEDIUM,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            ANTIALIASED_QUALITY,
            FF_SWISS,
            face.as_ptr(),
        )
    }
}

/// Measures exact physical pixel width of a localized tooltip string using Windows Segoe UI
pub fn measure_tooltip_text_width_px(text: &str, scale_factor: f64) -> f32 {
    if text.is_empty() {
        return 0.0;
    }
    let wide: Vec<u16> = text.encode_utf16().collect();
    unsafe {
        let hdc = CreateCompatibleDC(std::ptr::null_mut());
        if hdc.is_null() {
            return (text.chars().count() as f32) * 6.5 * (scale_factor as f32);
        }
        let hfont = create_tooltip_font(scale_factor);
        let old_font = if !hfont.is_null() { SelectObject(hdc, hfont) } else { std::ptr::null_mut() };
        let mut sz = SIZE::default();
        let ok = GetTextExtentPoint32W(hdc, wide.as_ptr(), wide.len() as i32, &mut sz);
        if !hfont.is_null() {
            SelectObject(hdc, old_font);
            DeleteObject(hfont);
        }
        DeleteDC(hdc);
        if ok != 0 && sz.cx > 0 {
            sz.cx as f32
        } else {
            (text.chars().count() as f32) * 6.5 * (scale_factor as f32)
        }
    }
}

/// Composites crisp grayscale-antialiased Segoe UI white text over premultiplied BGRA tooltip pill
pub fn composite_tooltip_text_bgra(
    bgra: &mut [u8],
    canvas_w: u32,
    canvas_h: u32,
    pill: &DockTooltipPillRect,
    text: &str,
    scale_factor: f64,
) {
    if text.is_empty() || pill.alpha <= 0.01 {
        return;
    }
    let px0 = pill.x.floor().max(0.0) as i32;
    let py0 = pill.y.floor().max(0.0) as i32;
    let pw = pill.width.ceil().max(1.0) as i32;
    let ph = pill.height.ceil().max(1.0) as i32;
    let wide: Vec<u16> = text.encode_utf16().collect();
    unsafe {
        let hdc = CreateCompatibleDC(std::ptr::null_mut());
        if hdc.is_null() {
            return;
        }
        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmi_header.bi_size = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmi_header.bi_width = pw;
        bmi.bmi_header.bi_height = -ph;
        bmi.bmi_header.bi_planes = 1;
        bmi.bmi_header.bi_bit_count = 32;
        bmi.bmi_header.bi_compression = BI_RGB;
        let mut scratch_bits: *mut u8 = std::ptr::null_mut();
        let hbmp = CreateDIBSection(hdc, &bmi, DIB_RGB_COLORS, &mut scratch_bits, std::ptr::null_mut(), 0);
        if hbmp.is_null() || scratch_bits.is_null() {
            if !hbmp.is_null() {
                DeleteObject(hbmp);
            }
            DeleteDC(hdc);
            return;
        }
        let old_bmp = SelectObject(hdc, hbmp);
        std::ptr::write_bytes(scratch_bits, 0, (pw * ph * 4) as usize);
        let hfont = create_tooltip_font(scale_factor);
        let old_font = if !hfont.is_null() { SelectObject(hdc, hfont) } else { std::ptr::null_mut() };
        SetBkMode(hdc, TRANSPARENT_BK);
        SetTextColor(hdc, 0x00FFFFFF);
        let mut rc = RECT { left: 0, top: 0, right: pw, bottom: ph };
        DrawTextW(hdc, wide.as_ptr(), wide.len() as i32, &mut rc, DT_SINGLELINE | DT_CENTER | DT_VCENTER | DT_NOPREFIX);
        let scratch_slice = std::slice::from_raw_parts(scratch_bits, (pw * ph * 4) as usize);
        let cw = canvas_w as i32;
        let ch = canvas_h as i32;
        for ly in 0..ph {
            let dy = py0 + ly;
            if dy < 0 || dy >= ch {
                continue;
            }
            for lx in 0..pw {
                let dx = px0 + lx;
                if dx < 0 || dx >= cw {
                    continue;
                }
                let sidx = ((ly * pw + lx) * 4) as usize;
                let cov = scratch_slice[sidx].max(scratch_slice[sidx + 1]).max(scratch_slice[sidx + 2]);
                if cov == 0 {
                    continue;
                }
                let src_a = ((cov as f32) * pill.alpha.clamp(0.0, 1.0)).round().clamp(0.0, 255.0) as u32;
                if src_a == 0 {
                    continue;
                }
                let didx = ((dy * cw + dx) * 4) as usize;
                let inv = 255 - src_a;
                bgra[didx] = (src_a + ((bgra[didx] as u32 * inv + 127) / 255)).min(255) as u8;
                bgra[didx + 1] = (src_a + ((bgra[didx + 1] as u32 * inv + 127) / 255)).min(255) as u8;
                bgra[didx + 2] = (src_a + ((bgra[didx + 2] as u32 * inv + 127) / 255)).min(255) as u8;
                bgra[didx + 3] = (src_a + ((bgra[didx + 3] as u32 * inv + 127) / 255)).min(255) as u8;
            }
        }
        if !hfont.is_null() {
            SelectObject(hdc, old_font);
            DeleteObject(hfont);
        }
        SelectObject(hdc, old_bmp);
        DeleteObject(hbmp);
        DeleteDC(hdc);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_tooltip_text_measure_and_composite_vietnamese() {
        let width = measure_tooltip_text_width_px("Ghim khung chat", 1.0);
        assert!(width > 40.0 && width < 180.0, "unexpected text width={}", width);
        let cw = 200u32;
        let ch = 60u32;
        let mut bgra = vec![0u8; (cw * ch * 4) as usize];
        let pill = DockTooltipPillRect { x: 40.0, y: 18.0, width: 120.0, height: 24.0, alpha: 1.0 };
        composite_tooltip_text_bgra(&mut bgra, cw, ch, &pill, "Ẩn bong bóng", 1.0);
        let lit_pixels = bgra.chunks_exact(4).filter(|px| px[2] > 180 && px[3] > 180).count();
        assert!(lit_pixels > 20, "expected crisp antialiased glyphs, found {} lit pixels", lit_pixels);
    }
}
