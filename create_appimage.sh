#!/bin/bash
# create_appimage.sh - Script to create AppImage for AEP Downgrader

set -e  # Exit on any error

echo "Creating AppImage for AEP Downgrader..."

# Create AppDir structure
APPDIR="AEP-Downgrader.AppDir"
APPIMAGE_NAME="AEP-Downgrader-Linux.AppImage"

# Clean previous build
rm -rf "$APPDIR" "$APPIMAGE_NAME"

# Create directory structure
mkdir -p "$APPDIR"/usr/bin
mkdir -p "$APPDIR"/usr/share/applications
mkdir -p "$APPDIR"/usr/share/icons/hicolor/256x256/apps

# Copy the executable
cp dist/AEP-Downgrader "$APPDIR"/usr/bin/

# Create desktop file
cat > "$APPDIR/usr/share/applications/io.github.itsanchorpoint.AEPDowngrader.desktop" << EOF
[Desktop Entry]
Name=AEP Downgrader
Exec=AEP-Downgrader
Icon=aep-downgrader
Type=Application
Categories=Graphics;
Comment=Convert Adobe After Effects projects from newer to older versions
Terminal=false
EOF

# Create symlink for app run
ln -sf usr/bin/AEP-Downgrader "$APPDIR/AEP-Downgrader"

# Create symlink for icon (we'll use a generic application icon for now)
ln -sf /usr/share/icons/hicolor/256x256/apps/gimp-icon.png "$APPDIR/aep-downgrader.png" 2>/dev/null || true

# If linuxdeploy is available, use it
if command -v linuxdeploy &> /dev/null; then
    echo "Using linuxdeploy to create AppImage..."
    linuxdeploy --appdir="$APPDIR" --executable="$APPDIR/usr/bin/AEP-Downgrader" --output=appimage
else
    # Fallback: try to download and use linuxdeploy
    if [ ! -f "linuxdeploy-x86_64.AppImage" ]; then
        echo "Downloading linuxdeploy..."
        wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
        chmod +x linuxdeploy-x86_64.AppImage
    fi
    
    echo "Using downloaded linuxdeploy to create AppImage..."
    ./linuxdeploy-x86_64.AppImage --appdir="$APPDIR" --executable="$APPDIR/usr/bin/AEP-Downgrader" --output=appimage
fi

# Move the created AppImage to a standard name
if [ -f "AEP-Downgrader-x86_64.AppImage" ]; then
    mv "AEP-Downgrader-x86_64.AppImage" "$APPIMAGE_NAME"
fi

echo "AppImage created: $APPIMAGE_NAME"
ls -lh "$APPIMAGE_NAME"

echo "Build completed successfully!"