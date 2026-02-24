#!/usr/bin/env python3
"""
AEP Downgrader - Modern GUI Application with Dark Theme
Converts Adobe After Effects project files from newer versions to older ones

Copyright (C) 2024-2025  AEP Downgrader Contributors

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
"""

import sys
import os
import shutil
import struct
from pathlib import Path

from PyQt5.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QPushButton, QLabel, QComboBox, QFileDialog, QTextEdit,
    QProgressBar, QGroupBox, QFormLayout, QMessageBox, QFrame,
    QLineEdit, QCheckBox, QSizePolicy
)
from PyQt5.QtCore import Qt, QThread, pyqtSignal, QSize
from PyQt5.QtGui import QFont, QColor, QPalette, QIcon


class ModernDarkTheme:
    """Class to define modern dark theme colors and styles"""
    
    BACKGROUND = "#1e1e1e"
    PANEL = "#2d2d30"
    HIGHLIGHT = "#0078d4"
    TEXT = "#ffffff"
    TEXT_SECONDARY = "#cccccc"
    BORDER = "#3e3e42"
    SUCCESS = "#4ec978"
    ERROR = "#f48771"
    WARNING = "#ffcc66"


class DowngradeWorker(QThread):
    """Worker thread for performing the downgrade operation"""
    progress_signal = pyqtSignal(str)
    finished_signal = pyqtSignal(bool, str)

    def __init__(self, input_path, output_path, target_version):
        super().__init__()
        self.input_path = input_path
        self.output_path = output_path
        self.target_version = target_version  # String like "AE 24.x", "AE 23.x", etc.

    def run(self):
        """Execute the downgrade operation in a separate thread"""
        try:
            self.progress_signal.emit(f"Starting conversion to {self.target_version}...")

            # Read the input file
            with open(self.input_path, 'rb') as f:
                content = bytearray(f.read())

            self.progress_signal.emit("Analyzing file headers...")

            # Verify this looks like a valid .aep file by checking the header signature
            if len(content) < 52:
                raise Exception("File too small to be a valid .aep file")

            # Extract head chunk data (20 bytes starting after the chunk header)
            head_data = content[32:52]  # 20 bytes of head chunk data

            # Extract the key distinguishing bytes
            current_sig = [head_data[1], head_data[3], head_data[4], head_data[5], head_data[6], head_data[7]]

            self.progress_signal.emit(f"File signature: {[f'0x{b:02x}' for b in current_sig]}")

            # Determine the target signature based on the target version
            target_sig = self.get_target_signature(self.target_version)
            if not target_sig:
                raise Exception(f"Unsupported target version: {self.target_version}")

            self.progress_signal.emit(f"Target signature: {[f'0x{b:02x}' for b in target_sig]}")

            # Determine the transformation path based on current and target signatures
            transformations = self.get_transformations(current_sig, target_sig)

            modifications = 0
            for offset, from_val, to_val in transformations:
                if offset < len(content) and content[offset] == from_val:
                    content[offset] = to_val
                    modifications += 1

            # Special handling for AE 22.x conversion - add warning
            if self.target_version == "AE 22.x":
                self.progress_signal.emit("WARNING: Converting to AE 22.x may result in compatibility issues due to structural differences.")
                self.progress_signal.emit("Consider using AE 23.x as target for better compatibility.")

            self.progress_signal.emit(f"Applied {modifications} modifications")
            self.progress_signal.emit("Writing converted file...")

            # Write the modified content to output file
            with open(self.output_path, 'wb') as f:
                f.write(content)

            self.progress_signal.emit("Conversion completed successfully!")
            self.finished_signal.emit(True, f"File converted successfully with {modifications} modifications")

        except Exception as e:
            error_msg = f"Error during conversion: {str(e)}"
            self.progress_signal.emit(error_msg)
            self.finished_signal.emit(False, error_msg)


    def get_target_signature(self, target_version):
        """Get the target signature based on the target version - universal algorithm"""
        # Extract version number from string like "AE 24.x"
        try:
            version = int(target_version.split()[1].split('.')[0])
        except (IndexError, ValueError):
            return None
        
        # Universal pattern for head_data[1]: 0x5b + (version - 20)
        # Example: AE 24 -> 0x5b + 4 = 0x5f
        head1 = 0x5b + (version - 20)
        
        # Heuristics for other bytes based on version patterns
        # These are based on analysis of known AE version signatures
        
        # head_data[3] - no clear linear pattern, use version-based heuristics
        if version <= 22:
            head3 = 0x2b  # AE 22.x pattern
        elif version == 23:
            head3 = 0x09  # AE 23.x
        elif version == 24:
            head3 = 0x05  # AE 24.x
        elif version == 25:
            head3 = 0x09  # AE 25.x
        elif version == 26:
            head3 = 0x02  # AE 26.x
        else:
            # For unknown versions, try to estimate based on newer pattern
            head3 = 0x02  # Default to newer pattern
        
        # head_data[4]: 0x0b for older versions (22-23), 0x0f for newer (24+)
        head4 = 0x0f if version >= 24 else 0x0b
        
        # head_data[5] - complex pattern
        if version == 22:
            head5 = 0x33
        elif version == 23:
            head5 = 0x3b
        elif version == 24:
            head5 = 0x02
        elif version == 25:
            head5 = 0x0b
        elif version == 26:
            head5 = 0x10
        else:
            head5 = 0x10  # Default to newer pattern
        
        # head_data[6]: 0x06 for most versions, 0x86 for AE 24
        head6 = 0x86 if version == 24 else 0x06
        
        # head_data[7]: complex pattern
        if version == 22:
            head7 = 0x3b
        elif version == 23:
            head7 = 0x37
        elif version == 24:
            head7 = 0x34
        elif version == 25:
            head7 = 0x65
        elif version == 26:
            head7 = 0x43
        else:
            head7 = 0x43  # Default to newer pattern
        
        return [head1, head3, head4, head5, head6, head7]

    def get_transformations(self, current_sig, target_sig):
        """Get the list of transformations needed to convert from current to target signature"""
        transformations = []

        # Get current and target versions
        current_version = self.signature_to_version(current_sig)
        target_version = self.signature_to_version(target_sig)
        
        # Universal pattern: head_data[1] = 0x5b + (version - 20)
        # To convert to a target version, we need to set head_data[1] accordingly
        if current_version and target_version:
            # Calculate target byte for head_data[1]
            target_head1 = 0x5b + (target_version - 20)
            current_head1 = current_sig[0]
            
            if current_head1 != target_head1:
                offset = 32 + 1  # head_data[1] is at position 1 in head_data, which is offset 32+1 in file
                transformations.append((offset, current_head1, target_head1))

        return transformations

    def signature_to_version(self, sig):
        """Convert signature to version number using universal pattern detection"""
        # Universal pattern: head_data[1] = 0x5b + (version - 20)
        # Formula: version = head_data[1] - 0x5b + 20
        # This works for AE 22+ (0x5d = AE 22, 0x5e = AE 23, etc.)
        if len(sig) >= 1:
            major_version_byte = sig[0]
            if major_version_byte >= 0x5d and major_version_byte <= 0x6a:
                return major_version_byte - 0x5b + 20
        return None


class AEPDowngraderGUI(QMainWindow):
    """Main GUI window for AEP Downgrader"""
    
    def __init__(self):
        super().__init__()
        self.init_ui()
        self.setup_connections()
        self.worker = None

    def get_resource_path(self, relative_path):
        """Get absolute path to resource, works for dev and for PyInstaller"""
        try:
            # PyInstaller creates a temp folder and stores path in _MEIPASS
            base_path = sys._MEIPASS
        except Exception:
            base_path = os.path.abspath(".")

        # For PyInstaller bundled app, resources are in the executable's directory
        bundled_path = os.path.join(base_path, relative_path)
        if os.path.exists(bundled_path):
            return bundled_path

        # For development, look in the project structure
        dev_path = os.path.join(os.path.dirname(__file__), '..', relative_path)
        if os.path.exists(dev_path):
            return dev_path

        # Return the original path if nothing else works
        return relative_path

    def init_ui(self):
        """Initialize the user interface"""
        self.setWindowTitle("AEP Downgrader")
        self.setGeometry(100, 100, 900, 700)
        self.setMinimumSize(800, 600)

        # Set window icon
        icon_path = self.get_resource_path('assets/icon.png')
        if os.path.exists(icon_path):
            self.setWindowIcon(QIcon(icon_path))

        # Apply dark theme
        self.apply_dark_theme()
        
        # Central widget
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        main_layout = QVBoxLayout(central_widget)
        
        # Header
        header_label = QLabel("AEP Downgrader")
        header_font = QFont()
        header_font.setPointSize(20)
        header_font.setBold(True)
        header_label.setFont(header_font)
        header_label.setAlignment(Qt.AlignCenter)
        header_label.setStyleSheet(f"color: {ModernDarkTheme.HIGHLIGHT}; margin: 20px;")
        
        subtitle_label = QLabel("Convert Adobe After Effects projects from newer to older versions")
        subtitle_font = QFont()
        subtitle_font.setPointSize(12)
        subtitle_label.setFont(subtitle_font)
        subtitle_label.setAlignment(Qt.AlignCenter)
        subtitle_label.setStyleSheet(f"color: {ModernDarkTheme.TEXT_SECONDARY}; margin-bottom: 30px;")
        
        # Input/Output section
        io_group = QGroupBox("File Selection")
        io_group.setStyleSheet(self.get_groupbox_style())
        io_layout = QVBoxLayout(io_group)

        # Input file selection
        input_label = QLabel("Input File(s):")
        input_label.setStyleSheet(f"color: {ModernDarkTheme.TEXT}; font-weight: bold;")
        io_layout.addWidget(input_label)

        input_layout = QHBoxLayout()
        input_layout.setSpacing(5)  # Reduce spacing between line edit and button
        self.input_line_edit = QLineEdit()
        self.input_line_edit.setPlaceholderText("Select input .aep file(s)... (multiple files supported)")
        self.input_line_edit.setStyleSheet(self.get_line_edit_style())
        self.input_line_edit.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Fixed)
        self.input_line_edit.setMinimumHeight(37)
        self.input_line_edit.setMaximumHeight(37)
        self.input_browse_btn = QPushButton("Browse")
        self.input_browse_btn.setStyleSheet(self.get_compact_button_style())
        self.input_browse_btn.setSizePolicy(QSizePolicy.Fixed, QSizePolicy.Fixed)
        self.input_browse_btn.setFixedWidth(90)
        self.input_browse_btn.setFixedHeight(37)
        self.input_browse_btn.setMaximumHeight(37)
        self.input_browse_btn.setMinimumHeight(37)
        input_layout.addWidget(self.input_line_edit)
        input_layout.addWidget(self.input_browse_btn)
        input_layout.setAlignment(Qt.AlignVCenter)  # Align items vertically centered
        io_layout.addLayout(input_layout)

        # Output file selection
        output_label = QLabel("Output Directory:")
        output_label.setStyleSheet(f"color: {ModernDarkTheme.TEXT}; font-weight: bold;")
        io_layout.addWidget(output_label)

        output_layout = QHBoxLayout()
        output_layout.setSpacing(5)  # Reduce spacing between line edit and button
        self.output_line_edit = QLineEdit()
        self.output_line_edit.setPlaceholderText("Save near original file (default) or specify location...")
        self.output_line_edit.setStyleSheet(self.get_line_edit_style())
        self.output_line_edit.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Fixed)
        self.output_line_edit.setMinimumHeight(37)
        self.output_line_edit.setMaximumHeight(37)
        self.output_browse_btn = QPushButton("Browse")
        self.output_browse_btn.setStyleSheet(self.get_compact_button_style())
        self.output_browse_btn.setSizePolicy(QSizePolicy.Fixed, QSizePolicy.Fixed)
        self.output_browse_btn.setFixedWidth(90)
        self.output_browse_btn.setFixedHeight(37)
        self.output_browse_btn.setMaximumHeight(37)
        self.output_browse_btn.setMinimumHeight(37)
        output_layout.addWidget(self.output_line_edit)
        output_layout.addWidget(self.output_browse_btn)
        output_layout.setAlignment(Qt.AlignVCenter)  # Align items vertically centered
        io_layout.addLayout(output_layout)
        
        # Conversion options
        options_group = QGroupBox("Conversion Options")
        options_group.setStyleSheet(self.get_groupbox_style())
        options_layout = QVBoxLayout(options_group)

        # Detected version label
        self.detected_version_label = QLabel("Detected version: Unknown")
        self.detected_version_label.setStyleSheet(f"color: {ModernDarkTheme.TEXT_SECONDARY}; font-style: italic;")
        options_layout.addWidget(self.detected_version_label)

        # Target versions checkboxes
        self.target_25_checkbox = QCheckBox("Convert to AE 25.x")
        self.target_25_checkbox.setStyleSheet(self.get_checkbox_style())
        self.target_24_checkbox = QCheckBox("Convert to AE 24.x")
        self.target_24_checkbox.setStyleSheet(self.get_checkbox_style())
        self.target_23_checkbox = QCheckBox("Convert to AE 23.x")
        self.target_23_checkbox.setStyleSheet(self.get_checkbox_style())

        options_layout.addWidget(self.target_25_checkbox)
        options_layout.addWidget(self.target_24_checkbox)
        options_layout.addWidget(self.target_23_checkbox)
        
        # Action buttons
        button_layout = QHBoxLayout()
        self.convert_btn = QPushButton("Convert File")
        self.convert_btn.setStyleSheet(self.get_primary_button_style())
        self.convert_btn.setMinimumHeight(40)
        self.cancel_btn = QPushButton("Cancel")
        self.cancel_btn.setStyleSheet(self.get_button_style())
        self.cancel_btn.setMinimumHeight(40)
        button_layout.addWidget(self.convert_btn)
        button_layout.addWidget(self.cancel_btn)
        
        # Progress and log
        progress_group = QGroupBox("Progress & Log")
        progress_group.setStyleSheet(self.get_groupbox_style())
        progress_layout = QVBoxLayout(progress_group)

        self.progress_bar = QProgressBar()
        self.progress_bar.setStyleSheet(self.get_progress_bar_style())
        self.progress_bar.setMaximumHeight(25)  # Limit the height of progress bar
        self.log_text_edit = QTextEdit()
        self.log_text_edit.setMaximumHeight(200)
        self.log_text_edit.setStyleSheet(self.get_text_edit_style())
        self.log_text_edit.setReadOnly(True)

        progress_layout.addWidget(self.progress_bar)
        progress_layout.addWidget(self.log_text_edit)
        
        # Add all widgets to main layout
        main_layout.addWidget(header_label)
        main_layout.addWidget(subtitle_label)
        main_layout.addWidget(io_group)
        main_layout.addWidget(options_group)
        main_layout.addLayout(button_layout)
        main_layout.addWidget(progress_group)
        
        # Add stretch to push everything up
        main_layout.addStretch()
    
    def setup_connections(self):
        """Setup signal connections"""
        self.input_browse_btn.clicked.connect(self.browse_input_files)
        self.output_browse_btn.clicked.connect(self.browse_output_file)
        self.convert_btn.clicked.connect(self.start_conversion)
        self.cancel_btn.clicked.connect(self.cancel_conversion)
    
    def apply_dark_theme(self):
        """Apply dark theme to the application"""
        palette = QPalette()
        palette.setColor(QPalette.Window, QColor(ModernDarkTheme.BACKGROUND))
        palette.setColor(QPalette.WindowText, QColor(ModernDarkTheme.TEXT))
        palette.setColor(QPalette.Base, QColor(ModernDarkTheme.PANEL))
        palette.setColor(QPalette.AlternateBase, QColor(ModernDarkTheme.BACKGROUND))
        palette.setColor(QPalette.ToolTipBase, QColor(ModernDarkTheme.BACKGROUND))
        palette.setColor(QPalette.ToolTipText, QColor(ModernDarkTheme.TEXT))
        palette.setColor(QPalette.Text, QColor(ModernDarkTheme.TEXT))
        palette.setColor(QPalette.Button, QColor(ModernDarkTheme.PANEL))
        palette.setColor(QPalette.ButtonText, QColor(ModernDarkTheme.TEXT))
        palette.setColor(QPalette.BrightText, QColor(ModernDarkTheme.HIGHLIGHT))
        palette.setColor(QPalette.Highlight, QColor(ModernDarkTheme.HIGHLIGHT))
        palette.setColor(QPalette.HighlightedText, QColor("#000000"))
        
        self.setPalette(palette)
        
        # Set application stylesheet
        self.setStyleSheet(f"""
            QMainWindow {{
                background-color: {ModernDarkTheme.BACKGROUND};
                color: {ModernDarkTheme.TEXT};
            }}
            QMenuBar {{
                background-color: {ModernDarkTheme.PANEL};
                color: {ModernDarkTheme.TEXT};
            }}
            QMenuBar::item:selected {{
                background-color: {ModernDarkTheme.BACKGROUND};
            }}
            QStatusBar {{
                background-color: {ModernDarkTheme.PANEL};
                color: {ModernDarkTheme.TEXT};
            }}
        """)
    
    def get_groupbox_style(self):
        """Get stylesheet for group boxes"""
        return f"""
            QGroupBox {{
                font-weight: bold;
                color: {ModernDarkTheme.HIGHLIGHT};
                border: 1px solid {ModernDarkTheme.BORDER};
                border-radius: 5px;
                margin-top: 1ex;
                padding-top: 10px;
            }}
            QGroupBox::title {{
                subcontrol-origin: margin;
                left: 10px;
                padding: 0 5px 0 5px;
            }}
        """

    def get_checkbox_style(self):
        """Get stylesheet for checkboxes"""
        return f"""
            QCheckBox {{
                spacing: 5px;
                color: {ModernDarkTheme.TEXT};
            }}
            QCheckBox:disabled {{
                color: #666666;
            }}
            QCheckBox::indicator {{
                width: 18px;
                height: 18px;
            }}
            QCheckBox::indicator:unchecked {{
                border: 1px solid {ModernDarkTheme.BORDER};
                background-color: {ModernDarkTheme.PANEL};
            }}
            QCheckBox::indicator:unchecked:disabled {{
                border: 1px solid #444444;
                background-color: #2a2a2a;
            }}
            QCheckBox::indicator:checked {{
                border: 1px solid {ModernDarkTheme.HIGHLIGHT};
                background-color: {ModernDarkTheme.HIGHLIGHT};
            }}
            QCheckBox::indicator:checked:disabled {{
                border: 1px solid #444444;
                background-color: #444444;
            }}
        """
    
    def get_line_edit_style(self):
        """Get stylesheet for line edits"""
        return f"""
            QLineEdit {{
                background-color: {ModernDarkTheme.PANEL};
                border: 1px solid {ModernDarkTheme.BORDER};
                border-radius: 4px;
                padding: 8px;
                color: {ModernDarkTheme.TEXT};
            }}
            QLineEdit:focus {{
                border: 1px solid {ModernDarkTheme.HIGHLIGHT};
            }}
        """
    
    def get_combobox_style(self):
        """Get stylesheet for combo boxes"""
        return f"""
            QComboBox {{
                background-color: {ModernDarkTheme.PANEL};
                border: 1px solid {ModernDarkTheme.BORDER};
                border-radius: 4px;
                padding: 8px;
                color: {ModernDarkTheme.TEXT};
                min-height: 25px;
            }}
            QComboBox:focus {{
                border: 1px solid {ModernDarkTheme.HIGHLIGHT};
            }}
            QComboBox::drop-down {{
                border-left: 1px solid {ModernDarkTheme.BORDER};
                border-top-right-radius: 4px;
                border-bottom-right-radius: 4px;
            }}
            QComboBox::down-arrow {{
                image: none;
                width: 12px;
                height: 12px;
            }}
            QComboBox QAbstractItemView {{
                background-color: {ModernDarkTheme.PANEL};
                color: {ModernDarkTheme.TEXT};
                selection-background-color: {ModernDarkTheme.HIGHLIGHT};
            }}
        """
    
    def get_button_style(self):
        """Get stylesheet for regular buttons"""
        return f"""
            QPushButton {{
                background-color: {ModernDarkTheme.PANEL};
                border: 1px solid {ModernDarkTheme.BORDER};
                border-radius: 4px;
                padding: 8px 16px;
                color: {ModernDarkTheme.TEXT};
                min-height: 30px;
            }}
            QPushButton:hover {{
                background-color: #3e3e42;
            }}
            QPushButton:pressed {{
                background-color: #2a2a2d;
            }}
            QPushButton:disabled {{
                background-color: #2a2a2d;
                color: #666666;
                border: 1px solid #333333;
            }}
        """

    def get_compact_button_style(self):
        """Get stylesheet for compact buttons"""
        return f"""
            QPushButton {{
                background-color: {ModernDarkTheme.PANEL};
                border: 1px solid {ModernDarkTheme.BORDER};
                border-radius: 4px;
                padding: 6px 2px;  /* Reduced horizontal padding to align with input fields */
                color: {ModernDarkTheme.TEXT};
                min-height: 37px;
                max-height: 37px;
            }}
            QPushButton:hover {{
                background-color: #3e3e42;
            }}
            QPushButton:pressed {{
                background-color: #2a2a2d;
            }}
            QPushButton:disabled {{
                background-color: #2a2a2d;
                color: #666666;
                border: 1px solid #333333;
            }}
        """
    
    def get_primary_button_style(self):
        """Get stylesheet for primary action buttons"""
        return f"""
            QPushButton {{
                background-color: {ModernDarkTheme.HIGHLIGHT};
                border: 1px solid {ModernDarkTheme.HIGHLIGHT};
                border-radius: 4px;
                padding: 10px 20px;
                color: white;
                font-weight: bold;
                min-height: 30px;
            }}
            QPushButton:hover {{
                background-color: #106ebe;
            }}
            QPushButton:pressed {{
                background-color: #005a9e;
            }}
            QPushButton:disabled {{
                background-color: #3e3e42;
                border: 1px solid #3e3e42;
                color: #666666;
            }}
        """
    
    def get_progress_bar_style(self):
        """Get stylesheet for progress bar"""
        return f"""
            QProgressBar {{
                border: 1px solid {ModernDarkTheme.BORDER};
                border-radius: 4px;
                text-align: center;
                color: {ModernDarkTheme.TEXT};
                height: 20px;
            }}
            QProgressBar::chunk {{
                background-color: {ModernDarkTheme.HIGHLIGHT};
                width: 20px;
            }}
        """
    
    def get_text_edit_style(self):
        """Get stylesheet for text edit"""
        return f"""
            QTextEdit {{
                background-color: {ModernDarkTheme.PANEL};
                border: 1px solid {ModernDarkTheme.BORDER};
                border-radius: 4px;
                padding: 8px;
                color: {ModernDarkTheme.TEXT};
            }}
        """
    
    def browse_input_file(self):
        """Open file dialog to select input file (single file)"""
        file_path, _ = QFileDialog.getOpenFileName(
            self, "Select Input AEP File", "", "AEP Files (*.aep);;All Files (*)"
        )
        if file_path:
            self.input_line_edit.setText(file_path)

            # Detect version of the loaded file
            detected_version_str, detected_version_num = self.detect_ae_version(file_path)
            detected_version_parts = detected_version_str.split()
            detected_version_only = detected_version_parts[0] if detected_version_parts else "Unknown"
            self.detected_version_label.setText(f"Detected versions: {detected_version_only}")

            # Update checkbox states based on detected version
            self.update_version_checkboxes(detected_version_num)

            # Auto-populate output directory if not set
            if not self.output_line_edit.text():
                path = Path(file_path)
                output_dir = path.parent
                self.output_line_edit.setText(str(output_dir))
                self.output_line_edit.setPlaceholderText(f"Save near original file ({output_dir})")

    def browse_input_files(self):
        """Open file dialog to select multiple input files"""
        file_paths, _ = QFileDialog.getOpenFileNames(
            self, "Select Input AEP Files", "", "AEP Files (*.aep);;All Files (*)"
        )
        if file_paths:
            # Display count of selected files
            self.input_line_edit.setText(f"{len(file_paths)} files selected: {', '.join([Path(fp).name for fp in file_paths[:3]])}{'...' if len(file_paths) > 3 else ''}")

            # Store the actual file paths for processing
            self.selected_input_files = file_paths

            # Auto-detect versions from all files and update UI accordingly
            if file_paths:
                detected_versions = set()  # Use set to store unique versions
                highest_version = 0
                highest_version_file = file_paths[0]

                for file_path in file_paths:
                    detected_version_str, detected_version_num = self.detect_ae_version(file_path)
                    if detected_version_num > 0:  # Valid version detected
                        # Extract the full version string (e.g., "AE 24.x") instead of just the first word
                        version_parts = detected_version_str.split()
                        if len(version_parts) >= 2:
                            detected_versions.add(f"{version_parts[0]} {version_parts[1]}")  # "AE 24.x"
                        elif len(version_parts) >= 1:
                            detected_versions.add(version_parts[0])
                        if detected_version_num > highest_version:
                            highest_version = detected_version_num
                            highest_version_file = file_path

                # Display detected versions
                if detected_versions:
                    # Extract version numbers and sort them in descending order
                    def extract_version_number(version_str):
                        # Extract the major version number from strings like "AE 24.x"
                        try:
                            # Split by space and take the second part ("24.x"), then split by dot and take first part ("24")
                            return int(version_str.split()[1].split('.')[0])
                        except (ValueError, IndexError):
                            return 0

                    sorted_versions = sorted(detected_versions, reverse=True, key=extract_version_number)
                    versions_list = ', '.join(sorted_versions)
                    self.detected_version_label.setText(f"Detected versions: {versions_list}")
                else:
                    self.detected_version_label.setText("Detected versions: Unknown")

                # Update checkbox states based on the highest detected version
                self.update_version_checkboxes(highest_version)

                # Auto-populate output directory if not set
                if not self.output_line_edit.text():
                    first_path = Path(highest_version_file)
                    output_dir = first_path.parent
                    self.output_line_edit.setText(str(output_dir))
                    self.output_line_edit.setPlaceholderText(f"Save near original files ({output_dir})")

    def update_version_checkboxes(self, detected_version):
        """Update checkbox states based on detected version"""
        # Disable checkboxes for versions equal or higher than detected version
        # (can only downgrade to lower versions)
        if detected_version > 25:
            self.target_25_checkbox.setEnabled(True)
        else:
            self.target_25_checkbox.setEnabled(False)
            self.target_25_checkbox.setChecked(False)

        if detected_version > 24:
            self.target_24_checkbox.setEnabled(True)
        else:
            self.target_24_checkbox.setEnabled(False)
            self.target_24_checkbox.setChecked(False)

        if detected_version > 23:
            self.target_23_checkbox.setEnabled(True)
        else:
            self.target_23_checkbox.setEnabled(False)
            self.target_23_checkbox.setChecked(False)

    def detect_ae_version(self, file_path):
        """Detect the AE version of an .aep file based on header analysis"""
        try:
            with open(file_path, 'rb') as f:
                content = f.read()

            if len(content) < 52:
                return "Unknown (file too small)", 0

            # Extract head chunk data (20 bytes starting after the chunk header)
            head_data = content[32:52]  # 20 bytes of head chunk data

            # Extract the key distinguishing byte (head_data[1])
            # Pattern: head_data[1] = 0x5b + version_offset where version_offset starts at 2 for AE 22
            # Formula: version = head_data[1] - 0x5b + 20
            # Example: 0x5d (93) - 0x5b (91) + 20 = 22
            major_version_byte = head_data[1]
            
            # Check if this looks like a valid AE version (>= AE 22)
            if major_version_byte >= 0x5d and major_version_byte <= 0x6a:  # AE 22 to AE 33
                version = major_version_byte - 0x5b + 20
                return f"AE {version}.x (detected)", version
            
            return "Unknown version", 0
        except Exception as e:
            return f"Error: {str(e)}", 0
    
    def browse_output_file(self):
        """Open file dialog to select output directory"""
        directory = QFileDialog.getExistingDirectory(
            self, "Select Output Directory", ""
        )
        if directory:
            self.output_line_edit.setText(directory)
    
    def start_conversion(self):
        """Start the conversion process"""
        input_path = self.input_line_edit.text().strip()

        if not input_path:
            QMessageBox.warning(self, "Warning", "Please select an input file")
            return

        # Determine if we're dealing with multiple files
        if hasattr(self, 'selected_input_files') and self.selected_input_files:
            input_files = self.selected_input_files
        else:
            # Single file mode
            if not os.path.exists(input_path):
                QMessageBox.critical(self, "Error", f"Input file does not exist: {input_path}")
                return

            # Check if input is an .aep file
            if not input_path.lower().endswith('.aep'):
                QMessageBox.critical(self, "Error", "Input file must be an .aep file")
                return

            input_files = [input_path]

        # Get selected target versions
        target_versions = []
        if self.target_25_checkbox.isEnabled() and self.target_25_checkbox.isChecked():
            target_versions.append("AE 25.x")
        if self.target_24_checkbox.isEnabled() and self.target_24_checkbox.isChecked():
            target_versions.append("AE 24.x")
        if self.target_23_checkbox.isEnabled() and self.target_23_checkbox.isChecked():
            target_versions.append("AE 23.x")

        if not target_versions:
            QMessageBox.warning(self, "Warning", "Please select at least one target version")
            return

        # Disable UI during conversion
        self.convert_btn.setEnabled(False)
        self.cancel_btn.setEnabled(True)
        self.input_browse_btn.setEnabled(False)
        self.output_browse_btn.setEnabled(False)

        # Reset progress
        self.progress_bar.setValue(0)
        self.log_text_edit.clear()

        # Create and start worker threads for each file and target version combination
        self.active_workers = []  # Track active workers

        # Get output directory
        output_text = self.output_line_edit.text().strip()
        if output_text:
            output_dir = Path(output_text)
            if not output_dir.exists() or not output_dir.is_dir():
                QMessageBox.critical(self, "Error", f"Output directory does not exist: {output_dir}")
                self.reset_ui()
                return
        else:
            # If no output directory specified, use the directory of the first input file
            first_input_dir = Path(input_files[0]).parent
            output_dir = first_input_dir

        # Create workers for each file and each target version
        for input_file in input_files:
            input_path_obj = Path(input_file)

            # Determine output directory for this specific file
            if output_text:  # If user specified an output directory
                current_output_dir = output_dir
            else:  # Use the directory of the input file
                current_output_dir = input_path_obj.parent

            for target_version in target_versions:
                # Generate output filename based on input name and target version
                version_suffix = target_version.replace(".", "").replace(" ", "")  # "AE 24x"
                output_filename = f"{input_path_obj.stem}_{version_suffix}.aep"
                output_path = current_output_dir / output_filename

                # Create worker for this conversion
                worker = DowngradeWorker(str(input_file), str(output_path), target_version)
                worker.progress_signal.connect(self.update_log)
                worker.finished_signal.connect(self.single_conversion_finished)
                self.active_workers.append(worker)

        # Start all workers
        self.total_workers = len(self.active_workers)
        self.completed_workers = 0
        self.successful_conversions = 0

        if self.total_workers == 0:
            QMessageBox.warning(self, "Warning", "No conversions to perform")
            self.reset_ui()
            return

        for worker in self.active_workers:
            worker.start()

        self.update_log(f"Started {self.total_workers} conversion(s) for {len(input_files)} file(s)")

    def single_conversion_finished(self, success, message):
        """Handle completion of a single conversion"""
        self.completed_workers += 1

        if success:
            self.successful_conversions += 1

        if self.completed_workers >= self.total_workers:
            # All conversions finished
            self.all_conversions_finished()

    def all_conversions_finished(self):
        """Handle completion of all conversions"""
        self.update_log(f"All conversions completed. {self.successful_conversions}/{self.total_workers} successful.")

        if self.successful_conversions > 0:
            QMessageBox.information(
                self,
                "Success",
                f"Conversion completed!\n{self.successful_conversions}/{self.total_workers} files converted successfully."
            )
        else:
            QMessageBox.critical(
                self,
                "Error",
                "All conversions failed!"
            )

        self.reset_ui()
    
    def cancel_conversion(self):
        """Cancel the conversion process"""
        # Cancel all active workers
        if hasattr(self, 'active_workers') and self.active_workers:
            for worker in self.active_workers:
                if worker.isRunning():
                    worker.terminate()
                    worker.wait()

        self.reset_ui()
        self.update_log("Conversion cancelled by user")
    
    def update_log(self, message):
        """Update the log text area"""
        self.log_text_edit.append(message)
        # Scroll to bottom
        scrollbar = self.log_text_edit.verticalScrollBar()
        scrollbar.setValue(scrollbar.maximum())
    
    def conversion_finished(self, success, message):
        """Handle completion of conversion"""
        if success:
            self.progress_bar.setValue(100)
            self.update_log("✓ " + message)
            QMessageBox.information(self, "Success", f"Conversion completed successfully!\n\n{message}")
        else:
            self.update_log("✗ " + message)
            QMessageBox.critical(self, "Error", f"Conversion failed:\n\n{message}")
        
        self.reset_ui()
    
    def reset_ui(self):
        """Reset UI to initial state"""
        self.convert_btn.setEnabled(True)
        self.cancel_btn.setEnabled(False)
        self.input_browse_btn.setEnabled(True)
        self.output_browse_btn.setEnabled(True)
        # Clear worker references
        self.active_workers = None
        self.worker = None


def main():
    app = QApplication(sys.argv)
    
    # Set application properties
    app.setApplicationName("AEP Downgrader")
    app.setApplicationVersion("1.0")
    
    # Create and show main window
    window = AEPDowngraderGUI()
    window.show()
    
    sys.exit(app.exec_())


if __name__ == "__main__":
    main()