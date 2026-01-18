# AEP Downgrader

**AEP Downgrader** is a modern utility with a graphical interface for downgrading Adobe After Effects project files from newer versions to older ones.

## Features

- 🎨 **Modern GUI** with dark theme
- 🔧 **Precise conversion** at the binary header level
- 📊 **Supports conversion** from AE 25.x to AE 24.x and AE 23.x
- 🧪 **Based on analysis** of real .aep files from different versions

## How it Works

The program analyzes specific bytes in .aep file headers that differ between AE versions:
- In AE 25.x: bytes in head chunk have values [0x60, 0x01, 0x0f, 0x08, 0x86, 0x44] at positions [1, 3, 4, 5, 6, 7]
- In AE 24.x: bytes in head chunk have values [0x5f, 0x05, 0x0f, 0x02, 0x86, 0x34] at positions [1, 3, 4, 5, 6, 7]
- In AE 23.x: bytes in head chunk have values [0x5e, 0x09, 0x0b, 0x3b, 0x06, 0x37] at positions [1, 3, 4, 5, 6, 7]

The program modifies these bytes to bring the file to the format of an older version.

## Installation and Launch

### Requirements

- Python 3.7+
- PyQt5

### Installing Dependencies

```bash
pip install PyQt5
```

### Launching the Application

```bash
python src/AEPdowngrader.py
```

## Usage

1. Launch the application
2. Select an input .aep file (version 25.x, 24.x, or 23.x)
3. The application will automatically detect the file version
4. Select target versions for conversion (only lower versions are available)
5. Click "Convert File"
6. Wait for the process to complete

## Supported Conversions

- AE 25.x → AE 24.x (fully compatible)
- AE 25.x → AE 23.x (fully compatible)
- AE 24.x → AE 23.x (fully compatible)

## Examples

Example files are located in the `examples/` directory:
- `examples/dev_aeps/TEST22.x.aep` - AE 22.x project (for reference)
- `examples/dev_aeps/TEST23.x.aep` - AE 23.x project (for reference)
- `examples/dev_aeps/TEST24.x.aep` - AE 24.x project (for reference)
- `examples/dev_aeps/TEST25.x.aep` - AE 25.x project (for reference)

These example files were used during development to analyze differences between AE versions and validate the conversion process.

## Project Structure

```
├── src/                 # Application source code
│   └── AEPdowngrader.py # Main application file
├── examples/           # Example files for testing
│   └── dev_aeps/       # .aep files from different versions
├── assets/             # Assets (icons, images)
├── docs/               # Documentation
├── README.md          # Documentation
└── setup.py           # Package installation file
```

## Acknowledgments

This project is partially based on research conducted in the [aep-parser](https://github.com/uwe-mayer/aep-parser) repository, which provided valuable insights into .aep file structure.

## Cross-Platform Distribution

AEP Downgrader is available as a portable application for multiple platforms:

- **Windows**: Portable ZIP archive with executable
- **macOS**: DMG disk image with .app bundle
- **Linux**: AppImage or portable ZIP with executable

The application is built using PyInstaller to create standalone executables that bundle all necessary dependencies.

### Building from Source

To build the application yourself:

1. Clone the repository
2. Create a virtual environment: `python -m venv venv`
3. Activate it: `source venv/bin/activate` (Linux/macOS) or `venv\Scripts\activate` (Windows)
4. Install dependencies: `pip install pyinstaller PyQt5`
5. Build: `pyinstaller src/AEPdowngrader.py --onefile --windowed --name AEP-Downgrader`

Platform-specific build instructions are available in:
- BUILD_WINDOWS.md
- BUILD_MACOS.md
- DISTRIBUTION.md

### Creating Releases

To create a new release with cross-platform binaries:

1. Update the version in relevant files
2. Commit and push your changes
3. Create a new tag: `git tag v1.2.3`
4. Push the tag: `git push origin v1.2.3`
5. GitHub Actions will automatically build and create a release with binaries for all platforms

The release will include binaries for Windows, macOS, and Linux that users can download directly from the Releases page.

## Contributing

Feel free to submit issues and pull requests. For major changes, please open an issue first to discuss what you would like to change.

## Icon Requirements

The application supports custom icons. To add an icon:

1. Create an icon in PNG format with transparency
2. Recommended sizes: 256x256, 512x512, or 1024x1024 pixels (higher resolution icons will be downscaled as needed)
3. Save the icon as `assets/icon.png` in the project root
4. The application will automatically load and use this icon

For Windows distribution, you may also create an ICO file with multiple resolutions combined into a single file (`assets/icon.ico`).

## License

MIT