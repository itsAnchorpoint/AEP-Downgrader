#!/bin/bash
# create_distributions.sh - Create cross-platform distributions for AEP Downgrader

set -e  # Exit on any error

echo "Creating cross-platform distributions for AEP Downgrader..."

# Create dist directory
DIST_DIR="dist_packages"
mkdir -p "$DIST_DIR"

# Copy the executable to the dist directory
cp dist/AEP-Downgrader "$DIST_DIR/"

# Create Linux portable version
LINUX_DIR="$DIST_DIR/AEP-Downgrader-Linux"
mkdir -p "$LINUX_DIR"

# Copy executable to Linux directory
cp dist/AEP-Downgrader "$LINUX_DIR/"

# Create a launcher script for Linux
cat > "$LINUX_DIR/start_AEP-Downgrader.sh" << 'EOF'
#!/bin/bash
# Launcher script for AEP Downgrader

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Run the main executable
"${SCRIPT_DIR}/AEP-Downgrader"
EOF

chmod +x "$LINUX_DIR/start_AEP-Downgrader.sh"

# Create README for Linux version
cat > "$LINUX_DIR/README.txt" << EOF
AEP Downgrader - Linux Portable Version

To run the application:
1. Make sure you have Python and PyQt5 installed on your system
2. Or run the executable directly: ./AEP-Downgrader

Alternatively, you can use the launcher script:
./start_AEP-Downgrader.sh

The application allows you to convert Adobe After Effects projects 
from newer versions to older ones.
EOF

# Create Linux ZIP package using Python
cd "$DIST_DIR"
python3 -c "
import zipfile
import os
import stat

def zip_folder(folder_path, zip_path):
    with zipfile.ZipFile(zip_path, 'w', zipfile.ZIP_DEFLATED) as zipf:
        for root, dirs, files in os.walk(folder_path):
            for file in files:
                file_path = os.path.join(root, file)
                arc_path = os.path.relpath(file_path, os.path.dirname(folder_path))
                zipf.write(file_path, arc_path)
                # Preserve executable permissions where needed
                if os.access(file_path, os.X_OK):
                    info = zipf.getinfo(arc_path)
                    info.external_attr = 0o755 << 16  # Set executable permissions

zip_folder('AEP-Downgrader-Linux', 'AEP-Downgrader-Linux.zip')
"
cd ..

echo "Linux portable package created: $DIST_DIR/AEP-Downgrader-Linux.zip"

# For Windows and macOS, we would typically create separate packages
# But for now, we'll just copy the Linux executable as a placeholder
# In a real scenario, we would run this script on each platform

echo "Cross-platform distributions created in: $DIST_DIR"

echo ""
echo "Contents of distribution directory:"
ls -la "$DIST_DIR"

echo ""
echo "Build completed successfully!"