use std::{
    error::Error,
    ffi::OsStr,
    fs::File,
    io::{self, ErrorKind, Read},
    mem::{self, MaybeUninit},
    os::windows::ffi::OsStrExt,
};

use base64::{Engine, engine::general_purpose};
use image::RgbaImage;
use windows::{
    Win32::{
        Graphics::Gdi::{
            BI_RGB, HBITMAP, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC,
            GetDIBits, GetObjectW, HDC, HGDIOBJ, ReleaseDC,
        },
        Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
        UI::{
            Shell::{SHFILEINFOW, SHGFI_ICON, SHGetFileInfoW},
            WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON},
        },
    },
    core::PCWSTR,
};

struct ScopedDc(HDC);

impl Drop for ScopedDc {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                ReleaseDC(None, self.0);
            }
        }
    }
}

struct AutoBitmap(HBITMAP);

impl Drop for AutoBitmap {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = DeleteObject(HGDIOBJ::from(self.0));
            }
        }
    }
}

struct AutoIcon(HICON);

impl Drop for AutoIcon {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = DestroyIcon(self.0);
            }
        }
    }
}

pub unsafe fn get_hicon(file_path: &str) -> Result<HICON, Box<dyn Error>> {
    let wide_path: Vec<u16> = OsStr::new(file_path).encode_wide().chain(Some(0)).collect();
    let mut shfileinfo = MaybeUninit::<SHFILEINFOW>::uninit();

    let result = unsafe {
        SHGetFileInfoW(
            PCWSTR::from_raw(wide_path.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(shfileinfo.as_mut_ptr()),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON,
        )
    };

    if result == 0 {
        return Err(Box::new(io::Error::new(
            ErrorKind::Other,
            format!("failed to get hIcon for the file: {file_path}."),
        )));
    }

    let shfileinfo = unsafe { shfileinfo.assume_init() };
    let hicon = shfileinfo.hIcon;

    if hicon.is_invalid() {
        return Err(Box::new(io::Error::new(
            ErrorKind::Other,
            format!("hIcon is invalid."),
        )));
    }

    Ok(shfileinfo.hIcon)
}

pub unsafe fn hicon_to_image(icon: HICON) -> Result<RgbaImage, Box<dyn Error>> {
    let bitmap_size_i32 = i32::try_from(mem::size_of::<BITMAP>())?;
    let biheader_size_u32 = u32::try_from(mem::size_of::<BITMAPINFOHEADER>())?;

    let mut info = MaybeUninit::uninit();
    unsafe {
        GetIconInfo(icon, info.as_mut_ptr())
            .map_err(|e| io::Error::new(ErrorKind::Other, format!("GetIconInfo failed: {e:?}")))
    }?;
    let info = unsafe { info.assume_init() };

    let _hbm_mask = AutoBitmap(info.hbmMask);
    let _hbm_color = AutoBitmap(info.hbmColor);
    let _icon_guard = AutoIcon(icon);

    let mut bitmap: MaybeUninit<BITMAP> = MaybeUninit::uninit();
    let result = unsafe {
        GetObjectW(
            HGDIOBJ::from(info.hbmColor),
            bitmap_size_i32,
            Some(bitmap.as_mut_ptr().cast()),
        )
    };
    if result != bitmap_size_i32 {
        return Err(Box::new(io::Error::new(
            ErrorKind::Other,
            format!("GetObjectW failed, expected {bitmap_size_i32}, got {result}"),
        )));
    }
    let bitmap = unsafe { bitmap.assume_init() };

    let width_abs = bitmap.bmWidth.unsigned_abs();
    let height_abs = bitmap.bmHeight.unsigned_abs();

    let width_u32 = u32::try_from(width_abs)?;
    let height_u32 = u32::try_from(height_abs)?;
    let width_usize = usize::try_from(width_abs)?;
    let height_usize = usize::try_from(height_abs)?;

    let buf_size = width_usize
        .checked_mul(height_usize)
        .ok_or_else(|| io::Error::new(ErrorKind::Other, "Buffer size overflow"))?;

    let mut buf = vec![0u32; buf_size];

    let dc = unsafe { GetDC(None) };
    if dc.is_invalid() {
        return Err(Box::new(io::Error::new(
            ErrorKind::Other,
            "GetDC returned null",
        )));
    }
    let _dc_guard = ScopedDc(dc);

    let mut bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: biheader_size_u32,
            biWidth: bitmap.bmWidth,
            biHeight: -bitmap.bmHeight,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [Default::default()],
    };
    let result = unsafe {
        GetDIBits(
            dc,
            info.hbmColor,
            0,
            height_u32,
            Some(buf.as_mut_ptr().cast()),
            &mut bitmap_info,
            DIB_RGB_COLORS,
        )
    };
    if result != bitmap.bmHeight {
        return Err(Box::new(io::Error::new(
            ErrorKind::Other,
            format!("GetDIBits failed, expected {height_u32}, got {result}"),
        )));
    }

    Ok(RgbaImage::from_fn(width_u32, height_u32, |x, y| {
        let idx = y as usize * width_usize + x as usize;
        let [b, g, r, a] = buf[idx].to_le_bytes();
        [r, g, b, a].into()
    }))
}

pub fn icon_to_image(icon_path: &str) -> Result<RgbaImage, Box<dyn Error>> {
    let mut file = File::open(icon_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let image = image::load_from_memory(&buffer)
        .map_err(|_| io::Error::new(ErrorKind::Other, format!("failed to decode image.")))?;
    Ok(image.to_rgba8())
}

pub fn icon_to_base64(icon_path: &str) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(icon_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(general_purpose::STANDARD.encode(&buffer))
}
