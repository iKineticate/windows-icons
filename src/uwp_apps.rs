use std::{
    error::Error,
    fs,
    io::{self, ErrorKind},
    path::Path,
};

use image::RgbaImage;

use crate::utils::image_utils::{icon_to_base64, icon_to_image};

pub fn get_uwp_icon(process_path: &str) -> Result<RgbaImage, Box<dyn Error>> {
    let icon_path = get_icon_file_path(process_path)?;
    let rgba_image = icon_to_image(&icon_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to get icon image for path: '{process_path}'\n{e}"),
        )
    })?;
    Ok(rgba_image)
}

pub fn get_uwp_icon_base64(process_path: &str) -> Result<String, Box<dyn Error>> {
    let icon_path = get_icon_file_path(process_path)?;
    let base64 = icon_to_base64(&icon_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to get icon base64 for path: '{process_path}'\n{e}"),
        )
    })?;
    Ok(base64)
}

fn get_icon_file_path(app_path: &str) -> Result<String, Box<dyn Error>> {
    if !Path::new(app_path).exists() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            format!("app path does not exist: {app_path}"),
        )));
    }

    let package_folder = Path::new(app_path).parent().ok_or_else(|| {
        io::Error::new(
            ErrorKind::NotFound,
            format!("failed to get parent directory: '{app_path}'"),
        )
    })?;

    // let icon_path_vec = ["assets/logo.png", "assets/logo.ico", "assets/icon.png", "assets/DesktopShortcut.ico", ...]
    let desktop_icon_path = package_folder.join("assets").join("DesktopShortcut.ico");
    if desktop_icon_path.exists() {
        return Ok(desktop_icon_path
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                io::Error::new(
                    ErrorKind::Other,
                    format!("failed to convert path to string: {:?}.", desktop_icon_path),
                )
            })?);
    } else {
        let manifest_path = package_folder.join("AppxManifest.xml");
        if manifest_path.exists() {
            let manifest_content = fs::read_to_string(&manifest_path).map_err(|_| {
                io::Error::new(ErrorKind::Other, "could not to read the AppxManifest.xml.")
            })?;

            let icon_path = extract_icon_path(&manifest_content)?;
            let icon_full_path = package_folder.join(icon_path);
            if icon_full_path.exists() {
                return Ok(icon_full_path
                    .to_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| {
                        io::Error::new(
                            ErrorKind::Other,
                            format!("failed to convert path to string: {:?}.", icon_full_path),
                        )
                    })?);
            } else {
                return Err(Box::new(io::Error::new(
                    ErrorKind::NotFound,
                    format!(
                        "icon path provided by the manifest does not exist: {:?}.",
                        icon_full_path
                    ),
                )));
            }
        } else {
            return Err(Box::new(io::Error::new(
                ErrorKind::NotFound,
                format!("AppxManifest.xml does not exist: {:?}.", manifest_path),
            )));
        }
    }
}

fn extract_icon_path(manifest_content: &str) -> Result<String, Box<dyn Error>> {
    // Look for the <Logo>...</Logo> tag in the manifest
    let start_tag = "<Logo>";
    let end_tag = "</Logo>";

    if let Some(start) = manifest_content.find(start_tag) {
        if let Some(end) = manifest_content.find(end_tag) {
            let start_pos = start + start_tag.len();
            let icon_path = &manifest_content[start_pos..end];
            return Ok(icon_path.trim().to_string());
        }
    }

    Err(Box::new(io::Error::new(
        ErrorKind::NotFound,
        "icon path not found in manifest.",
    )))
}
