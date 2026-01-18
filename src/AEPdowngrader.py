#!/usr/bin/env python3
"""
AEP Downgrader - Modern GUI Application with Dark Theme
Converts Adobe After Effects project files from newer versions to older ones
"""
import sys
import os
import shutil
from pathlib import Path

from PyQt5.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QPushButton, QLabel, QComboBox, QFileDialog, QTextEdit,
    QProgressBar, QGroupBox, QFormLayout, QMessageBox, QFrame,
    QLineEdit
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
    
    def __init__(self, input_path, output_path):
        super().__init__()
        self.input_path = input_path
        self.output_path = output_path
    
    def run(self):
        """Execute the downgrade operation in a separate thread"""
        try:
            self.progress_signal.emit("Starting conversion...")
            
            # Read the input file
            with open(self.input_path, 'rb') as f:
                content = bytearray(f.read())
            
            self.progress_signal.emit("Analyzing file headers...")
            
            # Verify this looks like an AE 25.x file by checking the header signature
            if len(content) < 52:
                raise Exception("File too small to be a valid .aep file")
            
            # Extract head chunk data (20 bytes starting after the chunk header)
            head_data = content[32:52]  # 20 bytes of head chunk data
            
            # AE 25.x signature: [0x60, 0x01, 0x0f, 0x08, 0x86, 0x44] at positions [1, 3, 4, 5, 6, 7]
            # AE 24.x signature: [0x5f, 0x05, 0x0f, 0x02, 0x86, 0x34] at positions [1, 3, 4, 5, 6, 7]
            
            ae_25_sig = [head_data[1], head_data[3], head_data[4], head_data[5], head_data[6], head_data[7]]
            ae_25_expected = [0x60, 0x01, 0x0f, 0x08, 0x86, 0x44]
            
            self.progress_signal.emit(f"File signature: {[f'0x{b:02x}' for b in ae_25_sig]}")
            
            # Check if this looks like an AE 25.x file
            if ae_25_sig != ae_25_expected:
                self.progress_signal.emit("Warning: File doesn't have the expected AE 25.x signature, but attempting conversion anyway...")
            
            # Define the transformations from AE 25.x to AE 24.x
            header_transformations = [
                (32 + 1, 0x60, 0x5f),  # head[1]: AE 25.x -> AE 24.x
                (32 + 3, 0x01, 0x05),  # head[3]: AE 25.x -> AE 24.x
                (32 + 5, 0x08, 0x02),  # head[5]: AE 25.x -> AE 24.x
                (32 + 7, 0x44, 0x34),  # head[7]: AE 25.x -> AE 24.x
            ]
            
            modifications = 0
            for offset, from_val, to_val in header_transformations:
                if offset < len(content) and content[offset] == from_val:
                    content[offset] = to_val
                    modifications += 1
            
            # Apply nhed transformations
            nhed_transformations = [
                (60 + 1, 0x01, 0x00),
                (60 + 16, 0x01, 0x00),
                (60 + 22, 0xfa, 0x00),
                (60 + 23, 0xbf, 0x91),
                (60 + 24, 0xb1, 0xd6),
                (60 + 28, 0xfa, 0x00),
                (60 + 29, 0xbf, 0x91),
                (60 + 30, 0xb1, 0xd6),
                (60 + 31, 0x40, 0x40),
            ]
            
            for offset, from_val, to_val in nhed_transformations:
                if offset < len(content):
                    if content[offset] == from_val:
                        content[offset] = to_val
                        modifications += 1
            
            # Apply nnhd transformations
            nnhd_transformations = [
                (100 + 1, 0x01, 0x00),
                (100 + 23, 0x01, 0x00),
                (100 + 30, 0xfa, 0x00),
                (100 + 31, 0xbf, 0x91),
                (100 + 32, 0xb1, 0xd6),
                (100 + 33, 0x41, 0x41),
                (100 + 36, 0xfa, 0x00),
                (100 + 37, 0xbf, 0x91),
                (100 + 38, 0xb1, 0xd6),
                (100 + 39, 0x40, 0x40),
            ]
            
            for offset, from_val, to_val in nnhd_transformations:
                if offset < len(content):
                    if content[offset] == from_val:
                        content[offset] = to_val
                        modifications += 1
            
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


class AEPDowngraderGUI(QMainWindow):
    """Main GUI window for AEP Downgrader"""
    
    def __init__(self):
        super().__init__()
        self.init_ui()
        self.setup_connections()
        self.worker = None
    
    def init_ui(self):
        """Initialize the user interface"""
        self.setWindowTitle("AEP Downgrader")
        self.setGeometry(100, 100, 800, 600)
        self.setMinimumSize(700, 500)
        
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
        io_layout = QFormLayout(io_group)
        
        # Input file selection
        input_layout = QHBoxLayout()
        self.input_line_edit = QLineEdit()
        self.input_line_edit.setPlaceholderText("Select input .aep file...")
        self.input_line_edit.setStyleSheet(self.get_line_edit_style())
        self.input_browse_btn = QPushButton("Browse")
        self.input_browse_btn.setStyleSheet(self.get_button_style())
        input_layout.addWidget(self.input_line_edit)
        input_layout.addWidget(self.input_browse_btn)
        
        # Output file selection
        output_layout = QHBoxLayout()
        self.output_line_edit = QLineEdit()
        self.output_line_edit.setPlaceholderText("Specify output .aep file...")
        self.output_line_edit.setStyleSheet(self.get_line_edit_style())
        self.output_browse_btn = QPushButton("Browse")
        self.output_browse_btn.setStyleSheet(self.get_button_style())
        output_layout.addWidget(self.output_line_edit)
        output_layout.addWidget(self.output_browse_btn)
        
        io_layout.addRow("Input File:", input_layout)
        io_layout.addRow("Output File:", output_layout)
        
        # Conversion options
        options_group = QGroupBox("Conversion Options")
        options_group.setStyleSheet(self.get_groupbox_style())
        options_layout = QFormLayout(options_group)
        
        self.version_combo = QComboBox()
        self.version_combo.addItems(["AE 25.x → AE 24.x"])
        self.version_combo.setStyleSheet(self.get_combobox_style())
        options_layout.addRow("Conversion:", self.version_combo)
        
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
        self.input_browse_btn.clicked.connect(self.browse_input_file)
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
        """Open file dialog to select input file"""
        file_path, _ = QFileDialog.getOpenFileName(
            self, "Select Input AEP File", "", "AEP Files (*.aep);;All Files (*)"
        )
        if file_path:
            self.input_line_edit.setText(file_path)
            # Auto-populate output if not set
            if not self.output_line_edit.text():
                path = Path(file_path)
                output_path = path.parent / f"{path.stem}_downgraded.aep"
                self.output_line_edit.setText(str(output_path))
    
    def browse_output_file(self):
        """Open file dialog to select output file"""
        file_path, _ = QFileDialog.getSaveFileName(
            self, "Select Output AEP File", "", "AEP Files (*.aep);;All Files (*)"
        )
        if file_path:
            self.output_line_edit.setText(file_path)
    
    def start_conversion(self):
        """Start the conversion process"""
        input_path = self.input_line_edit.text().strip()
        output_path = self.output_line_edit.text().strip()
        
        if not input_path:
            QMessageBox.warning(self, "Warning", "Please select an input file")
            return
        
        if not output_path:
            QMessageBox.warning(self, "Warning", "Please specify an output file")
            return
        
        if not os.path.exists(input_path):
            QMessageBox.critical(self, "Error", f"Input file does not exist: {input_path}")
            return
        
        # Check if input is an .aep file
        if not input_path.lower().endswith('.aep'):
            QMessageBox.critical(self, "Error", "Input file must be an .aep file")
            return
        
        # Check if output is an .aep file
        if not output_path.lower().endswith('.aep'):
            QMessageBox.critical(self, "Error", "Output file must be an .aep file")
            return
        
        # Disable UI during conversion
        self.convert_btn.setEnabled(False)
        self.cancel_btn.setEnabled(True)
        self.input_browse_btn.setEnabled(False)
        self.output_browse_btn.setEnabled(False)
        
        # Reset progress
        self.progress_bar.setValue(0)
        self.log_text_edit.clear()
        
        # Create and start worker thread
        self.worker = DowngradeWorker(input_path, output_path)
        self.worker.progress_signal.connect(self.update_log)
        self.worker.finished_signal.connect(self.conversion_finished)
        self.worker.start()
    
    def cancel_conversion(self):
        """Cancel the conversion process"""
        if self.worker and self.worker.isRunning():
            self.worker.terminate()
            self.worker.wait()
        
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