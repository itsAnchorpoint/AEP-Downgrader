#!/bin/bash
# Launcher script for AEP Downgrader

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Run the main executable
"${SCRIPT_DIR}/AEP-Downgrader"
