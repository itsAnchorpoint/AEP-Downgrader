#!/bin/bash
# build_app.sh - Script to build cross-platform distributions for AEP Downgrader

set -e  # Exit on any error

echo "Building AEP Downgrader cross-platform distributions..."

# Create dist directory
mkdir -p dist

# Build for current platform using PyInstaller
echo "Building executable for current platform..."
pyinstaller AEP-Downgrader.spec --clean

# Determine platform and create appropriate distribution
PLATFORM=$(uname -s)

if [[ "$PLATFORM" == "Linux" ]]; then
    echo "Creating AppImage for Linux..."
    
    # Install linuxdeploy if not present
    if ! command -v linuxdeploy &> /dev/null; then
        echo "Installing linuxdeploy..."
        wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
        chmod +x linuxdeploy-x86_64.AppImage
        sudo mv linuxdeploy-x86_64.AppImage /usr/local/bin/linuxdeploy
    fi
    
    # Create AppDir
    mkdir -p AEP-Downgrader.AppDir
    cp -r dist/AEP-Downgrader/* AEP-Downgrader.AppDir/
    
    # Create desktop file
    cat > AEP-Downgrader.AppDir/AEP-Downgrader.desktop << EOF
[Desktop Entry]
Name=AEP Downgrader
Exec=AEP-Downgrader
Icon=application-x-executable
Type=Application
Categories=Graphics;
Comment=Convert Adobe After Effects projects from newer to older versions
EOF
    
    # Create AppImage
    linuxdeploy --appdir AEP-Downgrader.AppDir --executable AEP-Downgrader.AppDir/AEP-Downgrader --output appimage
    mv *.AppImage dist/AEP-Downgrader-Linux.AppImage
    
elif [[ "$PLATFORM" == "Darwin" ]]; then
    echo "Creating DMG for macOS..."
    
    # Create DMG using hdiutil (macOS utility)
    cd dist
    ln -s /Applications ./Applications_link
    hdiutil create -volname "AEP-Downgrader" -srcfolder "AEP-Downgrader.app" -ov -format UDZO "AEP-Downgrader-macOS.dmg" || true
    rm Applications_link
    cd ..
    
else
    echo "Creating ZIP archive for Windows..."
    
    # Create ZIP for Windows portable version
    cd dist
    zip -r ../dist/AEP-Downgrader-Windows.zip AEP-Downgrader/
    cd ..
fi

echo "Build completed! Check the dist/ directory for your distribution files."