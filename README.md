<div align="center">
  <h1>AEP Downgrader</h1>
  <p><strong>Convert Adobe After Effects projects from newer to older versions</strong></p>

  [![License](https://img.shields.io/github/license/itsAnchorpoint/AEP-Downgrader)](LICENSE)
  [![Release](https://img.shields.io/github/v/release/itsAnchorpoint/AEP-Downgrader)](https://github.com/itsAnchorpoint/AEP-Downgrader/releases)
  [![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)](https://github.com/itsAnchorpoint/AEP-Downgrader/releases)

  <img src="assets/icon.png" alt="AEP Downgrader Logo" width="200" height="200">
</div>

---

## About

AEP Downgrader converts Adobe After Effects project files (.aep) from newer versions to older ones. Useful for team workflows where members use different AE versions.

Features a modern dark-themed UI built with PyQt5.

## Supported Conversions

| From | To |
|------|-----|
| AE 26.x | 25.x, 24.x, 23.x |
| AE 25.x | 24.x, 23.x |
| AE 24.x | 23.x |
| AE 23.x | 22.x |

## Download

Pre-built binaries available on the [Releases page](https://github.com/itsAnchorpoint/AEP-Downgrader/releases):

- **Windows**: `AEP-Downgrader-Windows.zip` → extract and run `AEP-Downgrader.exe`
- **macOS**: `AEP-Downgrader-macOS.dmg` → open and drag app to Applications
- **Linux**: `AEP-Downgrader-Linux.tar.gz` → extract and run `./AEP-Downgrader`

## Usage

1. Launch the application
2. Select one or more .aep files
3. The app automatically detects the source version
4. Select target version(s) using checkboxes
5. Click "Convert"
6. Converted files are saved in the same folder with version suffix (e.g., `project_AE24x.aep`)

## Debug Mode

Built-in debug logging for troubleshooting:

1. Menu: **Debug** → **Enable Debug Mode** (or press `Ctrl+D`)
2. View logs: **Debug** → **View Debug Logs**
3. Export report: **Debug** → **Export Debug Report**

Debug mode is included in all builds - no additional installation required.

## Building from Source

### Prerequisites
- Python 3.9+
- pip

### Build

```bash
# Clone and enter directory
git clone https://github.com/itsAnchorpoint/AEP-Downgrader.git
cd AEP-Downgrader

# Create virtual environment
python -m venv venv
source venv/bin/activate  # Linux/macOS
# venv\Scripts\activate  # Windows

# Install dependencies
pip install pyinstaller PyQt5 psutil

# Build
pyinstaller src/AEPdowngrader.py --onefile --windowed --name AEP-Downgrader --add-data "assets:assets" --hidden-import=psutil --hidden-import=debug_logger --collect-all=PyQt5
```

For detailed platform-specific instructions, see:
- [BUILD_WINDOWS.md](BUILD_WINDOWS.md)
- [BUILD_MACOS.md](BUILD_MACOS.md)

## Development

### Running tests
```bash
# Activate venv first
source venv/bin/activate
python src/AEPdowngrader.py
```

### Creating a release

```bash
# Update version in src/AEPdowngrader.py (lines 896, 1549)
# Update version in setup.py

git commit -m "Release v1.2.0"
git tag v1.2.0
git push origin main
git push origin v1.2.0
```

GitHub Actions automatically builds all platforms and creates the release.

## License

GNU General Public License v3.0 - see [LICENSE](LICENSE) for details.
