# macOS Build Instructions

## Prerequisites
- Python 3.7+
- pip
- Xcode command line tools

## Building

1. Create a virtual environment:
   ```bash
   python3 -m venv venv
   source venv/bin/activate
   ```

2. Install dependencies:
   ```bash
   pip install pyinstaller PyQt5
   ```

3. Build the application bundle:
   ```bash
   pyinstaller src/AEPdowngrader.py --onedir --windowed --name AEP-Downgrader --icon=assets/icon.icns
   ```

4. Create distribution package:
   ```bash
   # The application bundle will be in dist/AEP-Downgrader.app
   # Create DMG for distribution
   hdiutil create -volname "AEP-Downgrader" -srcfolder dist/AEP-Downgrader.app -ov -format UDZO AEP-Downgrader-macOS.dmg
   ```

## Alternative: One-file executable
```bash
pyinstaller src/AEPdowngrader.py --onefile --windowed --name AEP-Downgrader
```

## Result
This creates a macOS application bundle that can be distributed as a DMG file for easy installation to the Applications folder.