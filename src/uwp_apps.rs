use std::{
    error::Error, ffi::OsStr, fs, io::{self, ErrorKind}, path::Path
};

use image::RgbaImage;
use glob::glob;

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
                .map(|s| s.to_owned())
                .ok_or_else(|| {
                    io::Error::new(
                        ErrorKind::Other,
                        format!("failed to convert path to string: {:?}.", icon_full_path),
                    )
                })?);
        } else {
            find_matching_logo_file(&icon_full_path, package_folder)
        }
    } else {
        fuzzy_get_icon_file_path(package_folder).map_err(|e| Box::new(io::Error::new(ErrorKind::Other, format!("AppxManifest.xml does not exist and {e}"))) as Box<dyn Error>)
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

fn find_matching_logo_file(icon_full_path: &Path, package_folder: &Path) -> Result<String, Box<dyn Error>> {
    let parent_path = icon_full_path.parent().ok_or_else(|| io::Error::new(
        ErrorKind::NotFound,
        "no directory found"
    ))?;
    
    let filter_name = icon_full_path.file_stem().ok_or_else(|| io::Error::new(
        ErrorKind::NotFound,
        "no file name found"
    ))?;

    let extension = icon_full_path.extension().ok_or_else(|| io::Error::new(
        ErrorKind::NotFound,
        "no extension found"
    ))?;

    // '*' might be ".scale-size", and may include theme characters "contrast-white" and "contrast-black"
    let pattern = format!("{}/{}*.{}", parent_path.display(), filter_name.to_string_lossy(), extension.to_string_lossy());
    let exclude_theme = ["contrast-white", "contrast-black"];
    let mut matching_logo_files = Vec::new();

    for logo_path in glob(&pattern)?.filter_map(Result::ok) {
        if logo_path.is_file() {
            if let Some(file_name) = logo_path.file_stem().and_then(OsStr::to_str) {
                if !exclude_theme.iter().any(|&n| file_name.trim().to_lowercase().contains(n)) {
                    let metadata = fs::metadata(&logo_path).map_err(|_| io::Error::new(
                        ErrorKind::NotFound,
                        "failed to get logo information"
                    ))?;
                    let logo_path = logo_path.to_string_lossy().into_owned();
                    let logo_size = metadata.len();
                    matching_logo_files.push((logo_path, logo_size));
                }
            }
        }
    }

    let max_size_logo_file_path = matching_logo_files
        .iter()
        .max_by_key(|(_, size)| size)
        .map(|(path, _)| path.to_owned());

    if let Some(path) = max_size_logo_file_path {
        Ok(path)
    } else {
        fuzzy_get_icon_file_path(package_folder)
    }
}

fn fuzzy_get_icon_file_path(package_folder: &Path) -> Result<String, Box<dyn Error>> {
    let matching_names = ["logo", "icon", "DesktopShortcut"];
    let matching_extension = ["png", "ico"];
    let mut matching_logo_files = Vec::new();

    for name in matching_names {
        for ext in matching_extension {
            let pattern = format!("{}/**/{}.{}", package_folder.display(), name, ext);
            for logo_path in glob(&pattern)?.filter_map(Result::ok) {
                if logo_path.is_file() {
                    let metadata = fs::metadata(&logo_path).map_err(|_| io::Error::new(
                        ErrorKind::NotFound,
                        format!("failed to get logo information")
                    ))?;
                    let logo_path = logo_path.to_string_lossy().into_owned();
                    let logo_size = metadata.len();
                    matching_logo_files.push((logo_path, logo_size));
                }
            }
        }
    }

    let max_size_logo_file_path = matching_logo_files
        .iter()
        .max_by_key(|(_, size)| size)
        .map(|(path, _)| path.to_owned())
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::NotFound,
            "no matching logo files found",
        ))?;

    Ok(max_size_logo_file_path)
}