use std::{
    error::Error,
    path::Path,
};

use base64::Engine as _;
use base64::engine::general_purpose;
use image::RgbaImage;
use utils::image_utils::{get_hicon, hicon_to_image};
use utils::process_utils::get_process_path;
use uwp_apps::{get_uwp_icon, get_uwp_icon_base64};

mod utils {
    pub mod image_utils;
    pub mod process_utils;
}
mod uwp_apps;

fn is_uwp_app(path: &Path) -> bool {
    let is_uwp = path
        .to_string_lossy()
        .contains("Programme/WindowsApps");

    let is_wsa = path
        .to_string_lossy()
        .contains("WindowsSubsystemForAndroid");

    is_uwp && !is_wsa
}

pub fn get_icon_by_path<P: AsRef<Path>>(path: P) -> Result<RgbaImage, Box<dyn Error>> {
    let path = path.as_ref();
    if is_uwp_app(path) {
        get_uwp_icon(path)
    } else {
        unsafe {
            let hicon = get_hicon(path)?;
            hicon_to_image(hicon)
        }
    }
}

pub fn get_icon_base64_by_path<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn Error>> {
    let path = path.as_ref();
    if is_uwp_app(path) {
        get_uwp_icon_base64(path)
    } else {
        let icon_image = get_icon_by_path(path)?;
        let mut buffer = Vec::with_capacity(1024 * 50);
        icon_image.write_to(
            &mut std::io::Cursor::new(&mut buffer),
            image::ImageFormat::Png,
        )?;
        Ok(general_purpose::STANDARD.encode(&buffer))
    }
}

pub fn get_icon_by_process_id(process_id: u32) -> Result<RgbaImage, Box<dyn Error>> {
    let process_path = get_process_path(process_id)?;
    get_icon_by_path(&process_path)
}

pub fn get_icon_base64_by_process_id(process_id: u32) -> Result<String, Box<dyn Error>> {
    let process_path = get_process_path(process_id)?;
    get_icon_base64_by_path(&process_path)
}
