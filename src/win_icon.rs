//! Icônes système Windows pour un chemin (fichier ou dossier), pour affichage egui.

use std::path::Path;

use egui::ColorImage;
use windows::core::{Interface, PCWSTR};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HGDIOBJ,
};
use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
use windows::Win32::UI::Shell::{
    IShellItem, IShellItemImageFactory, SHCreateItemFromParsingName, SHFILEINFOW, SHGFI_FLAGS,
    SHGFI_ADDOVERLAYS, SHGFI_ICON, SHGFI_SHELLICONSIZE, SHGFI_SMALLICON, SHGetFileInfoW,
    SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY, SIIGBF_SCALEUP,
};
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, HICON};

fn wide_null_path(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    
    let path_str = path.to_string_lossy();
    let cleaned = if path_str.starts_with(r"\\?\UNC\") {
        format!(r"\\{}", &path_str[8..])
    } else if path_str.starts_with(r"\\?\") {
        path_str[4..].to_string()
    } else {
        path_str.into_owned()
    };
    
    std::ffi::OsStr::new(&cleaned)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

unsafe fn get_dib_bits(hbmp: windows::Win32::Graphics::Gdi::HBITMAP) -> Option<(u32, u32, Vec<u8>)> {
    use windows::Win32::Graphics::Gdi::{GetObjectW, BITMAP, GetDC, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, GetDIBits, DIB_RGB_COLORS};
    let mut bm: BITMAP = std::mem::zeroed();
    if GetObjectW(
        windows::Win32::Graphics::Gdi::HGDIOBJ(hbmp.0),
        std::mem::size_of::<BITMAP>() as i32,
        Some(std::ptr::addr_of_mut!(bm).cast()),
    ) == 0 || bm.bmWidth <= 0 || bm.bmHeight == 0 {
        return None;
    }

    let w = bm.bmWidth as u32;
    let h = bm.bmHeight.abs() as u32;

    let hdc = GetDC(None);
    if hdc.is_invalid() { return None; }

    let mut bi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w as i32,
            biHeight: -(h as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0 as u32,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut pixels: Vec<u8> = vec![0; (w * h * 4) as usize];
    if GetDIBits(hdc, hbmp, 0, h, Some(pixels.as_mut_ptr().cast()), &mut bi, DIB_RGB_COLORS) == 0 {
        let _ = ReleaseDC(None, hdc);
        return None;
    }
    let _ = ReleaseDC(None, hdc);
    Some((w, h, pixels))
}

unsafe fn hbitmap_to_color_image(hbmp: windows::Win32::Graphics::Gdi::HBITMAP) -> Option<ColorImage> {
    hbitmap_to_color_image_with_mask(hbmp, windows::Win32::Graphics::Gdi::HBITMAP::default())
}

unsafe fn hbitmap_to_color_image_with_mask(hbmp: windows::Win32::Graphics::Gdi::HBITMAP, hmask: windows::Win32::Graphics::Gdi::HBITMAP) -> Option<ColorImage> {
    let (w, h, mut pixels) = get_dib_bits(hbmp)?;
    
    // Check if image actually uses the alpha channel
    let mut has_alpha = false;
    for i in (0..pixels.len()).step_by(4) {
        if pixels[i + 3] != 0 {
            has_alpha = true;
            break;
        }
    }

    // If it doesn't use alpha but has a mask, apply the mask
    if !has_alpha && !hmask.is_invalid() {
        if let Some((mw, mh, mask_pixels)) = get_dib_bits(hmask) {
            let actual_h = if mh == h * 2 { h } else { mh };
            if mw == w && actual_h == h {
                for y in 0..h {
                    for x in 0..w {
                        let i = ((y * w + x) * 4) as usize;
                        // For monochrome mask returned as 32bpp, white is transparent, black is opaque
                        let mask_r = mask_pixels[i + 2];
                        pixels[i + 3] = if mask_r > 128 { 0 } else { 255 };
                    }
                }
            }
        }
    } else if !has_alpha {
        // Fallback for no mask and no alpha => fully opaque
        for i in (0..pixels.len()).step_by(4) {
            pixels[i + 3] = 255;
        }
    }

    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for i in (0..pixels.len()).step_by(4) {
        rgba.extend_from_slice(&[
            pixels[i + 2], // R
            pixels[i + 1], // G
            pixels[i],     // B
            pixels[i + 3], // A
        ]);
    }
    Some(ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba))
}

unsafe fn hicon_to_color_image(icon: windows::Win32::UI::WindowsAndMessaging::HICON, _size: i32) -> Option<ColorImage> {
    use windows::Win32::UI::WindowsAndMessaging::{GetIconInfo, ICONINFO};
    use windows::Win32::Graphics::Gdi::{DeleteObject, HGDIOBJ};
    let mut info: ICONINFO = std::mem::zeroed();
    if GetIconInfo(icon, &mut info).is_ok() {
        let mut img = None;
        if !info.hbmColor.is_invalid() {
            img = hbitmap_to_color_image_with_mask(info.hbmColor, info.hbmMask);
        } else if !info.hbmMask.is_invalid() {
            img = hbitmap_to_color_image_with_mask(info.hbmMask, windows::Win32::Graphics::Gdi::HBITMAP::default());
        }
        
        let _ = DeleteObject(HGDIOBJ(info.hbmColor.0));
        let _ = DeleteObject(HGDIOBJ(info.hbmMask.0));
        
        return img;
    }
    None
}

/// Icône via `IShellItemImageFactory` (qualité shell moderne).
pub fn shell_icon_for_path(path: &Path, size_px: i32) -> Option<ColorImage> {
    unsafe {
        let wide = wide_null_path(path);
        let item: IShellItem = SHCreateItemFromParsingName(PCWSTR(wide.as_ptr()), None).ok()?;
        let factory: IShellItemImageFactory = item.cast().ok()?;

        let hbmp = factory
            .GetImage(
                windows::Win32::Foundation::SIZE { cx: size_px, cy: size_px },
                SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK | SIIGBF_SCALEUP,
            )
            .ok()?;

        let img = hbitmap_to_color_image(hbmp);
        let _ = DeleteObject(HGDIOBJ(hbmp.0));
        img
    }
}

/// Repli : `SHGetFileInfo` + dessin de l’icône (fichiers et dossiers, avec overlays Shell).
pub fn shgetfile_icon(path: &Path, size_px: i32) -> Option<ColorImage> {
    let mut flags: SHGFI_FLAGS = SHGFI_ICON | SHGFI_ADDOVERLAYS | SHGFI_SHELLICONSIZE;
    if size_px <= 20 {
        flags |= SHGFI_SMALLICON;
    }

    unsafe {
        let wide = wide_null_path(path);
        let mut sfi: SHFILEINFOW = std::mem::zeroed();
        let ok = SHGetFileInfoW(
            PCWSTR(wide.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(std::ptr::addr_of_mut!(sfi)),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            flags,
        );
        if ok == 0 || sfi.hIcon.is_invalid() {
            return None;
        }
        let img = hicon_to_color_image(sfi.hIcon, size_px);
        let _ = DestroyIcon(sfi.hIcon);
        img
    }
}

use windows::Win32::System::Com::{CoInitialize, CoUninitialize, CoCreateInstance, CLSCTX_INPROC_SERVER, IPersistFile, STGM};
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
use windows::Win32::UI::WindowsAndMessaging::PrivateExtractIconsW;

pub fn extract_icon_file(path: &Path, size_px: i32) -> Option<ColorImage> {
    unsafe {
        let wide = wide_null_path(path);
        let mut path_arr = [0u16; 260];
        let len = wide.len().min(260);
        path_arr[..len].copy_from_slice(&wide[..len]);
        
        let mut hicons = [HICON::default(); 1];
        let mut iconids = [0u32; 1];
        
        let res = PrivateExtractIconsW(
            &path_arr,
            0,
            size_px,
            size_px,
            Some(&mut hicons),
            Some(iconids.as_mut_ptr()),
            0,
        );
        
        let hicon = hicons[0];
        if res > 0 && !hicon.is_invalid() {
            let img = hicon_to_color_image(hicon, size_px);
            let _ = DestroyIcon(hicon);
            return img;
        }
        None
    }
}

pub fn resolve_lnk_icon(path: &Path) -> Option<std::path::PathBuf> {
    unsafe {
        let _ = CoInitialize(None);
        let mut result = None;
        if let Ok(sl) = CoCreateInstance::<_, IShellLinkW>(&ShellLink, None, CLSCTX_INPROC_SERVER) {
            if let Ok(pf) = sl.cast::<IPersistFile>() {
                let wide = wide_null_path(path);
                if pf.Load(windows::core::PCWSTR(wide.as_ptr()), STGM::default()).is_ok() {
                    let mut icon_path = [0u16; 1024];
                    let mut icon_index = 0i32;
                    if sl.GetIconLocation(&mut icon_path, &mut icon_index).is_ok() {
                        let len = icon_path.iter().position(|&c| c == 0).unwrap_or(icon_path.len());
                        if len > 0 {
                            let pb = std::path::PathBuf::from(String::from_utf16_lossy(&icon_path[..len]));
                            if pb.exists() {
                                result = Some(pb);
                            }
                        }
                    }
                    if result.is_none() {
                        let mut target_path = [0u16; 1024];
                        let mut pfd = std::mem::zeroed();
                        if sl.GetPath(&mut target_path, &mut pfd, 0).is_ok() {
                            let len = target_path.iter().position(|&c| c == 0).unwrap_or(target_path.len());
                            if len > 0 {
                                let pb = std::path::PathBuf::from(String::from_utf16_lossy(&target_path[..len]));
                                if pb.exists() {
                                    result = Some(pb);
                                }
                            }
                        }
                    }
                }
            }
        }
        let _ = CoUninitialize();
        result
    }
}

pub fn icon_for_path(path: &Path, size_px: i32) -> Option<ColorImage> {
    if path.extension().and_then(|s| s.to_str()).unwrap_or("").eq_ignore_ascii_case("url") {
        if let Ok(bytes) = std::fs::read(path) {
            let content = String::from_utf8_lossy(&bytes);
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some(icon_path) = trimmed.strip_prefix("IconFile=") {
                    let icon_path = icon_path.trim_matches('"').trim();
                    let icon_path_buf = std::path::PathBuf::from(icon_path);
                    if let Some(img) = extract_icon_file(&icon_path_buf, size_px) {
                        return Some(img);
                    }
                    if let Some(img) = shell_icon_for_path(&icon_path_buf, size_px).or_else(|| shgetfile_icon(&icon_path_buf, size_px)) {
                        return Some(img);
                    }
                }
            }
        }
    }
    
    if path.extension().and_then(|s| s.to_str()).unwrap_or("").eq_ignore_ascii_case("lnk") {
        if let Some(icon_path_buf) = resolve_lnk_icon(path) {
            if let Some(img) = extract_icon_file(&icon_path_buf, size_px) {
                return Some(img);
            }
            if let Some(img) = shell_icon_for_path(&icon_path_buf, size_px).or_else(|| shgetfile_icon(&icon_path_buf, size_px)) {
                return Some(img);
            }
        }
    }
    
    extract_icon_file(path, size_px)
        .or_else(|| shell_icon_for_path(path, size_px))
        .or_else(|| shgetfile_icon(path, size_px))
}


