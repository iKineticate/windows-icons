# Windows Icons

A simple Rust library to extract icons from files and running processes on Windows platforms.

## Features

- Retrieve icons by file path or process id
- Save as a PNG or base64 encoded string

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
windows-icons = "0.2.1"
```

## Usage

```rust
// Get icon as an image from a file path
let icon = get_icon_image_by_path("C:\\Windows\\System32\\notepad.exe").unwrap();
icon.save("notepad.png").unwrap();

// Get icon as a base64 string from a file path
let base64 = get_icon_base64_by_path("C:\\Windows\\System32\\calc.exe").unwrap();
println!("Calculator icon: {}", base64);

// Get icon as an image from a process ID
let process_id = 1234;

let icon = get_icon_image_by_process_id(process_id).unwrap();
icon.save("process.png").unwrap();

// Get icon as a base64 encoded string from a process ID
let base64 = get_icon_base64_by_process_id(process_id).unwrap();
println!("Process {} icon: {}", process_id, base64);
```

For more examples, check the [`examples/main.rs`](examples/main.rs).

## Requirements

This library is designed to work on Windows systems only.

## License

This project is licensed under the MIT License - see the [`LICENSE`](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgements

This library uses the following crates:

- `image` for image processing
- `base64` for base64 encoding
- `glob` for matching file paths
- `windows` for Windows API interactions

## Disclaimer

This library uses unsafe Rust code to interact with the Windows API. While efforts have been made to ensure safety, use it at your own risk.
