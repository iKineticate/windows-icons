use std::{
    error::Error,
    ffi::OsStr,
    fs::File,
    io::{self, ErrorKind, Read},
    mem::{self, MaybeUninit},
    os::windows::ffi::OsStrExt,
    ptr,
};

use base64::{Engine, engine::general_purpose};
use image::RgbaImage;
use windows::{
    Win32::{
        Graphics::Gdi::{
            BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC,
            GetDIBits, GetObjectW, HDC, HGDIOBJ, ReleaseDC,
        },
        Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
        UI::{
            Shell::{SHFILEINFOW, SHGFI_ICON, SHGetFileInfoW},
            WindowsAndMessaging::{GetIconInfo, HICON},
        },
    },
    core::PCWSTR,
};

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
    let shfileinfo = unsafe { shfileinfo.assume_init() };

    if result == 0 {
        return Err(Box::new(io::Error::new(
            ErrorKind::Other,
            format!("failed to get hIcon for the file: {file_path}."),
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
            .map_err(|_| io::Error::new(ErrorKind::Other, "failed to get icon info."))
    }?;
    let info = unsafe { info.assume_init() };
    unsafe {
        DeleteObject(HGDIOBJ::from(info.hbmMask))
            .ok()
            .map_err(|_| io::Error::new(ErrorKind::Other, "failed to delete mask bitmap."))
    }?;

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
            "failed to get info for the object.",
        )));
    }
    let bitmap = unsafe { bitmap.assume_init() };

    let width_u32 = u32::try_from(bitmap.bmWidth)?;
    let height_u32 = u32::try_from(bitmap.bmHeight)?;
    let width_usize = usize::try_from(bitmap.bmWidth)?;
    let height_usize = usize::try_from(bitmap.bmHeight)?;
    let buf_size = width_usize
        .checked_mul(height_usize)
        .ok_or_else(|| io::Error::new(ErrorKind::Other, "buffer size calculation overflow."))?;

    let mut buf: Vec<u32> = Vec::with_capacity(buf_size);

    let dc = unsafe { GetDC(None) };
    if dc == HDC(ptr::null_mut()) {
        return Err(Box::new(io::Error::new(
            ErrorKind::Other,
            "failed to get a handle to the DC.",
        )));
    }

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
            "failed to get DIB bits.",
        )));
    }

    unsafe { buf.set_len(buf.capacity()) };

    if unsafe { ReleaseDC(None, dc) } != 1 {
        return Err(Box::new(io::Error::new(
            ErrorKind::Other,
            "failed to releases the DC.",
        )));
    };
    unsafe {
        DeleteObject(HGDIOBJ::from(info.hbmColor))
            .ok()
            .map_err(|_| io::Error::new(ErrorKind::Other, "failed to delete color bitmap."))
    }?;

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
