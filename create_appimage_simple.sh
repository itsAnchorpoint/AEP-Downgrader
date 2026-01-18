#!/bin/bash
# create_appimage_simple.sh - Simple AppImage creation script for AEP Downgrader

set -e  # Exit on any error

echo "Creating AppImage for AEP Downgrader..."

# Define variables
APPNAME="AEP-Downgrader"
APPDIR="${APPNAME}.AppDir"
APPIMAGE_NAME="${APPNAME}-Linux.AppImage"

# Clean previous build
rm -rf "$APPDIR" "$APPIMAGE_NAME"

# Create AppDir structure
mkdir -p "$APPDIR"/usr/bin
mkdir -p "$APPDIR"/usr/share/applications
mkdir -p "$APPDIR"/usr/share/icons/hicolor/256x256/apps

# Copy the executable
cp dist/AEP-Downgrader "$APPDIR"/usr/bin/

# Create desktop file
cat > "$APPDIR"/AEP-Downgrader.desktop << EOF
[Desktop Entry]
Name=AEP Downgrader
Exec=AEP-Downgrader
Icon=application-x-executable
Type=Application
Categories=Graphics;
Comment=Convert Adobe After Effects projects from newer to older versions
Terminal=false
EOF

# Create symlinks
ln -sf usr/bin/AEP-Downgrader "$APPDIR/AEP-Downgrader"
ln -sf AEP-Downgrader.desktop "$APPDIR"/usr/share/applications/io.github.itsanchorpoint.AEPDowngrader.desktop

# Create AppRun script
cat > "$APPDIR/AppRun" << 'EOF'
#!/bin/sh
HERE="$(dirname "$(readlink -f "${0}")")"
export QT_QPA_PLATFORM_PLUGIN_PATH="$HERE/usr/plugins"
export QT_PLUGIN_PATH="$HERE/usr/plugins"
exec "$HERE/usr/bin/AEP-Downgrader" "$@"
EOF

chmod +x "$APPDIR/AppRun"

echo "AppDir structure created."

# Check if appimagetool is available
if command -v appimagetool &> /dev/null; then
    echo "Using appimagetool to create AppImage..."
    appimagetool "$APPDIR" "$APPIMAGE_NAME"
elif [ -f "appimagetool-x86_64.AppImage" ]; then
    echo "Using downloaded appimagetool..."
    ./appimagetool-x86_64.AppImage "$APPDIR" "$APPIMAGE_NAME"
else
    echo "appimagetool not found. Downloading..."
    wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
    chmod +x appimagetool-x86_64.AppImage
    ./appimagetool-x86_64.AppImage "$APPDIR" "$APPIMAGE_NAME"
fi

echo "AppImage created: $APPIMAGE_NAME"
ls -lh "$APPIMAGE_NAME"

echo "Build completed successfully!"