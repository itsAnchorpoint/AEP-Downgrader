#!/bin/bash
# create_appimage_with_icon.sh - Script to create AppImage with proper icon

set -e  # Exit on any error

echo "Creating AppImage with icon..."

# Install linuxdeploy if not present
if ! command -v linuxdeploy &> /dev/null; then
    echo "Installing linuxdeploy..."
    wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
    chmod +x linuxdeploy-x86_64.AppImage
    sudo mv linuxdeploy-x86_64.AppImage /usr/local/bin/linuxdeploy
fi

# Create AppDir structure
mkdir -p AEP-Downgrader.AppDir/usr/bin
mkdir -p AEP-Downgrader.AppDir/usr/share/icons/hicolor/512x512/apps
mkdir -p AEP-Downgrader.AppDir/usr/share/applications

# Copy the executable
cp dist/AEP-Downgrader AEP-Downgrader.AppDir/usr/bin/

# Copy the icon
cp assets/icon.png AEP-Downgrader.AppDir/usr/share/icons/hicolor/512x512/apps/io.github.itsanchorpoint.AEPDowngrader.png

# Create desktop file
cat > AEP-Downgrader.AppDir/AEP-Downgrader.desktop << EOF
[Desktop Entry]
Name=AEP Downgrader
Exec=AEP-Downgrader
Icon=io.github.itsanchorpoint.AEPDowngrader
Type=Application
Categories=Graphics;
Comment=Convert Adobe After Effects projects from newer to older versions
Terminal=false
EOF

# Create symlink for AppRun
ln -sf usr/bin/AEP-Downgrader AEP-Downgrader.AppDir/AppRun

# Create AppImage
linuxdeploy --appdir AEP-Downgrader.AppDir --executable AEP-Downgrader.AppDir/usr/bin/AEP-Downgrader --icon-file=AEP-Downgrader.AppDir/usr/share/icons/hicolor/512x512/apps/io.github.itsanchorpoint.AEPDowngrader.png --desktop-file=AEP-Downgrader.AppDir/AEP-Downgrader.desktop -o appimage

# Move to dist
mv *.AppImage dist/AEP-Downgrader-Linux.AppImage || echo "AppImage creation may have failed"

echo "AppImage created successfully!"