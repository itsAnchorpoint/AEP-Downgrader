#!/usr/bin/env python3
# build_cross_platform.py - Cross-platform build script for AEP Downgrader

import os
import sys
import platform
import subprocess
import shutil
from pathlib import Path

def run_command(cmd, cwd=None):
    """Run a command and check for errors"""
    print(f"Running: {' '.join(cmd) if isinstance(cmd, list) else cmd}")
    result = subprocess.run(cmd, shell=isinstance(cmd, str), cwd=cwd)
    if result.returncode != 0:
        print(f"Command failed with return code {result.returncode}")
        sys.exit(result.returncode)
    return result

def build_windows():
    """Build Windows portable executable"""
    print("Building for Windows...")
    
    # Run PyInstaller with spec file
    run_command(["pyinstaller", "AEP-Downgrader.spec", "--clean"])
    
    # Create portable distribution
    dist_path = Path("dist") / "AEP-Downgrader"
    zip_path = dist_path.parent / "AEP-Downgrader-Windows.zip"
    
    # Create ZIP archive
    shutil.make_archive(str(dist_path.parent / "AEP-Downgrader-Windows"), 'zip', dist_path)
    
    print(f"Windows build completed: {zip_path}")

def build_macos():
    """Build macOS .app and .dmg"""
    print("Building for macOS...")
    
    # Run PyInstaller with spec file
    run_command(["pyinstaller", "AEP-Downgrader.spec", "--clean"])
    
    app_source = Path("dist") / "AEP-Downgrader.app"
    dmg_output = Path("dist") / "AEP-Downgrader-macOS.dmg"
    
    # Create DMG using hdiutil
    if shutil.which("hdiutil"):
        # Create temporary directory for DMG creation
        temp_dir = Path("temp_dmg")
        temp_dir.mkdir(exist_ok=True)
        
        # Copy app to temp directory
        temp_app = temp_dir / "AEP-Downgrader.app"
        shutil.copytree(app_source, temp_app)
        
        # Create Applications link
        (temp_dir / "Applications").symlink_to("/Applications")
        
        # Create DMG
        run_command([
            "hdiutil", "create", "-volname", "AEP-Downgrader", 
            "-srcfolder", str(temp_dir), "-ov", "-format", "UDZO",
            str(dmg_output)
        ])
        
        # Cleanup
        shutil.rmtree(temp_dir)
        
        print(f"macOS build completed: {dmg_output}")
    else:
        print("hdiutil not found. Creating .app only.")
        print(f"macOS build completed: {app_source}")

def build_linux():
    """Build Linux AppImage"""
    print("Building for Linux...")
    
    # Run PyInstaller with spec file
    run_command(["pyinstaller", "AEP-Downgrader.spec", "--clean"])
    
    # Install linuxdeploy if not present
    if not shutil.which("linuxdeploy"):
        print("Installing linuxdeploy...")
        run_command("wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage -O linuxdeploy.AppImage")
        run_command("chmod +x linuxdeploy.AppImage")
        run_command("sudo mv linuxdeploy.AppImage /usr/local/bin/linuxdeploy")
    
    # Create AppDir
    appdir_path = Path("AEP-Downgrader.AppDir")
    if appdir_path.exists():
        shutil.rmtree(appdir_path)
    
    appdir_path.mkdir(exist_ok=True)
    
    # Copy built application to AppDir
    source_path = Path("dist") / "AEP-Downgrader"
    for item in source_path.iterdir():
        dest_item = appdir_path / item.name
        if item.is_dir():
            shutil.copytree(item, dest_item)
        else:
            shutil.copy2(item, dest_item)
    
    # Create desktop file
    desktop_content = """[Desktop Entry]
Name=AEP Downgrader
Exec=AEP-Downgrader
Icon=application-x-executable
Type=Application
Categories=Graphics;
Comment=Convert Adobe After Effects projects from newer to older versions
"""
    with open(appdir_path / "AEP-Downgrader.desktop", "w") as f:
        f.write(desktop_content)
    
    # Create AppImage
    run_command([
        "linuxdeploy", 
        "--appdir", str(appdir_path), 
        "--executable", str(appdir_path / "AEP-Downgrader"),
        "--output", "appimage"
    ])
    
    # Move AppImage to dist
    appimage_files = list(Path(".").glob("*.AppImage"))
    if appimage_files:
        target_path = Path("dist") / f"AEP-Downgrader-Linux.AppImage"
        shutil.move(str(appimage_files[0]), str(target_path))
        print(f"Linux build completed: {target_path}")
    else:
        print("AppImage creation failed!")

def main():
    system = platform.system().lower()
    
    print(f"Detected platform: {system}")
    
    # Create dist directory
    dist_path = Path("dist")
    dist_path.mkdir(exist_ok=True)
    
    if system == "windows":
        build_windows()
    elif system == "darwin":  # macOS
        build_macos()
    elif system == "linux":
        build_linux()
    else:
        print(f"Unsupported platform: {system}")
        sys.exit(1)

if __name__ == "__main__":
    main()