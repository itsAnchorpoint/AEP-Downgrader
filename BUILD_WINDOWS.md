# Windows Build Instructions

## Prerequisites
- Python 3.7+
- pip
- Git Bash or similar shell environment

## Building

1. Create a virtual environment:
   ```cmd
   python -m venv venv
   venv\Scripts\activate
   ```

2. Install dependencies:
   ```cmd
   pip install pyinstaller PyQt5 psutil
   ```

3. Build the executable:
   ```cmd
   pyinstaller src/AEPdowngrader.py --onefile --windowed --name AEP-Downgrader --icon=assets/icon.ico --add-data "assets:assets" --hidden-import=psutil --hidden-import=debug_logger --collect-all=PyQt5 --collect-all=debug_logger
   ```

4. Create distribution package:
   ```cmd
   # Create a directory for the portable app
   mkdir "AEP-Downgrader-Windows"
   copy dist\AEP-Downgrader.exe "AEP-Downgrader-Windows\"
   
   # Create README
   echo AEP Downgrader - Windows Portable Version > "AEP-Downgrader-Windows\README.txt"
   echo. >> "AEP-Downgrader-Windows\README.txt"
   echo To run the application, double-click AEP-Downgrader.exe >> "AEP-Downgrader-Windows\README.txt"
   
   # Create ZIP archive
   powershell -command "Compress-Archive -Path 'AEP-Downgrader-Windows' -DestinationPath 'AEP-Downgrader-Windows.zip'"
   ```

## Result
This creates a portable Windows application in a ZIP file that users can extract and run directly without installation.
