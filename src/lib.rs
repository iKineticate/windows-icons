use std::error::Error;

use base64::engine::general_purpose;
use base64::Engine as _;
use image::RgbaImage;
use utils::image_utils::{get_hicon, hicon_to_image};
use utils::process_utils::get_process_path;
use uwp_apps::{get_uwp_icon, get_uwp_icon_base64};

mod utils {
    pub mod image_utils;
    pub mod process_utils;
}
mod uwp_apps;

pub fn get_icon_by_process_id(process_id: u32) -> Result<RgbaImage, Box<dyn Error>> {
    let process_path =
        get_process_path(process_id).map_err(|e| format!("Failed to get process path: {e}."))?;
    get_icon_by_path(&process_path)
}

pub fn get_icon_base64_by_process_id(process_id: u32) -> Result<String, Box<dyn Error>> {
    let process_path =
        get_process_path(process_id).map_err(|e| format!("Failed to get process path: {e}."))?;
    get_icon_base64_by_path(&process_path)
}

pub fn get_icon_by_path(path: &str) -> Result<RgbaImage, Box<dyn Error>> {
    // Excluding ..\Users\MyUser\AppData\Local\Microsoft\WindowsApps and WSA applications.
    if path.contains("Program Files\\WindowsApps") && !path.contains("WindowsSubsystemForAndroid") {
        return get_uwp_icon(path);
    }

    unsafe {
        let hicon = get_hicon(path)?;
        hicon_to_image(hicon)
    }
}

pub fn get_icon_base64_by_path(path: &str) -> Result<String, Box<dyn Error>> {
    // Excluding ..\Users\MyUser\AppData\Local\Microsoft\WindowsApps and WSA applications.
    if path.contains("Program Files\\WindowsApps") && !path.contains("WindowsSubsystemForAndroid") {
        return get_uwp_icon_base64(path);
    }

    let icon_image = get_icon_by_path(path)?;
    let mut buffer = Vec::new();
    icon_image.write_to(
        &mut std::io::Cursor::new(&mut buffer),
        image::ImageFormat::Png,
    )?;
    Ok(general_purpose::STANDARD.encode(buffer))
}
