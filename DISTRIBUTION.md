# AEP Downgrader - Cross-Platform Distribution Guide

This document describes how to create cross-platform distributions for AEP Downgrader.

## Overview

AEP Downgrader is distributed as a portable application for multiple platforms:
- Windows: Portable ZIP with executable
- macOS: DMG disk image with .app bundle
- Linux: AppImage or portable ZIP with executable

## Building for Current Platform

To build the application for the current platform, use the following commands:

```bash
# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install pyinstaller PyQt5

# Build the executable
pyinstaller src/AEPdowngrader.py --onefile --windowed --name AEP-Downgrader
```

## Creating Platform-Specific Distributions

### Linux Distribution

The Linux distribution is created as a portable ZIP file containing:
- The executable application
- A launcher script
- Documentation

Use the `create_distributions.sh` script to create the Linux package.

### Windows Distribution

For Windows, the executable is packaged in a ZIP file with:
- The .exe file
- Any required DLLs
- Documentation

### macOS Distribution

For macOS, the application is packaged as:
- An .app bundle
- Distributed in a .dmg disk image

## Prerequisites for Building

- Python 3.7+
- pip
- Virtual environment support
- PyInstaller
- PyQt5

## Deployment Strategy

The application is designed to be portable and not require installation. Users can simply extract the archive and run the application directly.

For Linux, the application requires Qt libraries to be installed on the system. The PyInstaller build bundles Python and PyQt5 with the application to minimize external dependencies.

## Notes

- The application uses PyQt5 for its GUI
- The executable is built using PyInstaller
- Platform-specific packaging scripts automate the creation of distribution archives