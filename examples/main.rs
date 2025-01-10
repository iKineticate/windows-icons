use windows_icons::{
    get_icon_base64_by_path, get_icon_base64_by_process_id, get_icon_by_path,
    get_icon_by_process_id,
};

fn main() {
    let _ = std::fs::create_dir("output");

    let icon = get_icon_by_path("C:\\Windows\\System32\\notepad.exe").unwrap();
    icon.save("output/notepad.png").unwrap();

    let icon = get_icon_by_path("C:\\Windows\\System32\\calc.exe").unwrap();
    icon.save("output/calc.png").unwrap();

    let icon = get_icon_by_path("C:\\Windows\\System32\\cmd.exe").unwrap();
    icon.save("output/cmd.png").unwrap();

    let base64 = get_icon_base64_by_path("C:\\Windows\\System32\\notepad.exe").unwrap();
    println!("Notepad: {}", base64);

    let base64 = get_icon_base64_by_path("C:\\Windows\\System32\\calc.exe").unwrap();
    println!("Calc: {}", base64);

    let base64 = get_icon_base64_by_path("C:\\Windows\\System32\\cmd.exe").unwrap();
    println!("Cmd: {}", base64);

    // Substitute the process id to test
    let process_id = 0000;

    let base64 = get_icon_base64_by_process_id(process_id).unwrap();
    println!("Process {}: {}", process_id, base64);

    let icon = get_icon_by_process_id(process_id).unwrap();
    icon.save("output/process.png").unwrap();
}
