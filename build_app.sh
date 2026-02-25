#!/bin/bash
# build_app.sh - Script to build AEP Downgrader executable

set -e  # Exit on any error

echo "Building AEP Downgrader executable..."

# Create virtual environment if it doesn't exist
if [ ! -d "venv" ]; then
    python3 -m venv venv
fi

# Activate virtual environment
source venv/bin/activate

# Install dependencies
pip install pyinstaller PyQt5 psutil

# Check if icon exists and build accordingly
if [ -f "assets/icon.png" ]; then
    echo "Building with icon..."
    # Build the application with icon, including all necessary hidden imports
    pyinstaller src/AEPdowngrader.py \
        --onefile \
        --windowed \
        --name AEP-Downgrader \
        --icon=assets/icon.png \
        --add-data "assets:assets" \
        --hidden-import=psutil \
        --hidden-import=debug_logger \
        --collect-all=PyQt5 \
        --collect-all=debug_logger
else
    echo "Building without icon..."
    # Build the application without icon
    pyinstaller src/AEPdowngrader.py \
        --onefile \
        --windowed \
        --name AEP-Downgrader \
        --hidden-import=psutil \
        --hidden-import=debug_logger \
        --collect-all=PyQt5 \
        --collect-all=debug_logger
fi

echo "Build completed. The executable is located at dist/AEP-Downgrader"
