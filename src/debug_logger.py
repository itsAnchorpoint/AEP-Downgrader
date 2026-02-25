#!/usr/bin/env python3
"""
Debug Logging Module for AEP Downgrader
Provides comprehensive cross-platform debugging capabilities
"""

import sys
import os
import platform
import struct
import traceback
import datetime
import threading
import time
import json
import psutil
import tempfile
import shutil
from pathlib import Path
from io import StringIO
from datetime import datetime

# Try to import network-related modules
try:
    import urllib.request
    import urllib.error
    NETWORK_AVAILABLE = True
except ImportError:
    NETWORK_AVAILABLE = False


class DebugLevel:
    """Debug level constants"""
    TRACE = "TRACE"
    DEBUG = "DEBUG"
    INFO = "INFO"
    WARNING = "WARNING"
    ERROR = "ERROR"
    CRITICAL = "CRITICAL"


class PlatformInfo:
    """Cross-platform system information collector"""
    
    @staticmethod
    def get_platform_info():
        """Get comprehensive platform information"""
        info = {
            "system": platform.system(),
            "release": platform.release(),
            "version": platform.version(),
            "machine": platform.machine(),
            "processor": platform.processor(),
            "python_version": platform.python_version(),
            "python_implementation": platform.python_implementation(),
            "hostname": platform.node(),
        }
        
        # Platform-specific information
        if platform.system() == "Windows":
            info["windows_version"] = PlatformInfo._get_windows_version()
            info["windows_edition"] = PlatformInfo._get_windows_edition()
        elif platform.system() == "Darwin":  # macOS
            info["macos_version"] = PlatformInfo._get_macos_version()
        elif platform.system() == "Linux":
            info["linux_distribution"] = PlatformInfo._get_linux_distribution()
        
        return info
    
    @staticmethod
    def _get_windows_version():
        """Get Windows version details"""
        try:
            import winreg
            key = winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
            version = winreg.QueryValueEx(key, "CurrentBuild")[0]
            winreg.CloseKey(key)
            return version
        except Exception:
            return platform.release()
    
    @staticmethod
    def _get_windows_edition():
        """Get Windows edition"""
        try:
            import winreg
            key = winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
            edition = winreg.QueryValueEx(key, "EditionID")[0]
            winreg.CloseKey(key)
            return edition
        except Exception:
            return "Unknown"
    
    @staticmethod
    def _get_macos_version():
        """Get macOS version"""
        try:
            import subprocess
            result = subprocess.run(["sw_vers", "-productVersion"], 
                                  capture_output=True, text=True)
            return result.stdout.strip()
        except Exception:
            return platform.mac_ver()[0]
    
    @staticmethod
    def _get_linux_distribution():
        """Get Linux distribution info"""
        try:
            if os.path.exists("/etc/os-release"):
                with open("/etc/os-release") as f:
                    for line in f:
                        if line.startswith("PRETTY_NAME="):
                            return line.split("=")[1].strip().strip('"')
            return platform.linux_distribution()[0]
        except Exception:
            return "Unknown"


class MemoryInfo:
    """Memory and resource usage information collector"""
    
    @staticmethod
    def get_memory_info():
        """Get current memory usage information"""
        try:
            process = psutil.Process()
            mem_info = process.memory_info()
            
            info = {
                "rss_mb": round(mem_info.rss / (1024 * 1024), 2),
                "vms_mb": round(mem_info.vms / (1024 * 1024), 2),
                "percent": process.memory_percent(),
                "num_threads": process.num_threads(),
            }
            
            # Get system memory if available
            try:
                sys_mem = psutil.virtual_memory()
                info["system_total_mb"] = round(sys_mem.total / (1024 * 1024), 2)
                info["system_available_mb"] = round(sys_mem.available / (1024 * 1024), 2)
                info["system_percent"] = sys_mem.percent
            except Exception:
                pass
                
            return info
        except Exception as e:
            return {"error": str(e)}
    
    @staticmethod
    def get_cpu_info():
        """Get CPU usage information"""
        try:
            process = psutil.Process()
            return {
                "cpu_percent": process.cpu_percent(interval=0.1),
                "num_threads": process.num_threads(),
                "cpu_count": psutil.cpu_count(),
            }
        except Exception as e:
            return {"error": str(e)}


class FileSystemMonitor:
    """Monitor file system operations"""
    
    def __init__(self):
        self.operations = []
        self._lock = threading.Lock()
    
    def log_operation(self, operation_type, path, details=None):
        """Log a file system operation"""
        with self._lock:
            entry = {
                "timestamp": datetime.now().isoformat(),
                "type": operation_type,
                "path": str(path),
                "details": details or {},
                "thread_id": threading.get_ident(),
            }
            self.operations.append(entry)
    
    def log_read(self, path, size=None):
        """Log file read operation"""
        details = {"size": size} if size else {}
        self.log_operation("READ", path, details)
    
    def log_write(self, path, size=None):
        """Log file write operation"""
        details = {"size": size} if size else {}
        self.log_operation("WRITE", path, details)
    
    def log_delete(self, path):
        """Log file delete operation"""
        self.log_operation("DELETE", path)
    
    def log_exists(self, path):
        """Log file existence check"""
        self.log_operation("EXISTS", path, {"exists": os.path.exists(str(path))})
    
    def get_operations(self):
        """Get all logged operations"""
        with self._lock:
            return self.operations.copy()
    
    def clear(self):
        """Clear operation log"""
        with self._lock:
            self.operations.clear()


class NetworkMonitor:
    """Monitor network operations (if available)"""
    
    def __init__(self):
        self.requests = []
        self._lock = threading.Lock()
    
    def log_request(self, url, method="GET", headers=None, response_code=None, error=None):
        """Log a network request"""
        with self._lock:
            entry = {
                "timestamp": datetime.now().isoformat(),
                "url": str(url),
                "method": method,
                "headers": headers or {},
                "response_code": response_code,
                "error": str(error) if error else None,
                "thread_id": threading.get_ident(),
            }
            self.requests.append(entry)
    
    def get_requests(self):
        """Get all logged requests"""
        with self._lock:
            return self.requests.copy()
    
    def clear(self):
        """Clear request log"""
        with self._lock:
            self.requests.clear()


class DebugLogger:
    """Main debug logger class with cross-platform support"""
    
    _instance = None
    _lock = threading.Lock()
    
    def __new__(cls):
        if cls._instance is None:
            with cls._lock:
                if cls._instance is None:
                    cls._instance = super().__new__(cls)
        return cls._instance
    
    def __init__(self):
        if not hasattr(self, '_initialized'):
            self._initialized = True
            self.enabled = False
            self.log_file = None
            self.log_buffer = StringIO()
            self.platform_info = PlatformInfo.get_platform_info()
            self.fs_monitor = FileSystemMonitor()
            self.network_monitor = NetworkMonitor() if NETWORK_AVAILABLE else None
            self.start_time = None
            self.session_id = None
            self._buffer_lock = threading.Lock()
    
    def enable(self):
        """Enable debug mode"""
        self.enabled = True
        self.start_time = datetime.now()
        self.session_id = self._generate_session_id()
        
        # Create log file in temp directory
        log_dir = self._get_log_directory()
        log_path = os.path.join(log_dir, f"debug_{self.session_id}.log")
        self.log_file = open(log_path, 'w', encoding='utf-8')
        
        self._log(DebugLevel.INFO, "Debug mode enabled")
        self._log(DebugLevel.INFO, f"Session ID: {self.session_id}")
        self._log(DebugLevel.INFO, f"Platform: {self.platform_info['system']}")
        self._log(DebugLevel.INFO, f"Python: {self.platform_info['python_version']}")
        
        return log_path
    
    def disable(self):
        """Disable debug mode"""
        if self.enabled:
            self._log(DebugLevel.INFO, "Debug mode disabled")
            
            # Log summary
            self._log_summary()
            
            if self.log_file:
                self.log_file.close()
                self.log_file = None
            
            self.enabled = False
    
    def _generate_session_id(self):
        """Generate unique session ID"""
        return datetime.now().strftime("%Y%m%d_%H%M%S")
    
    def _get_log_directory(self):
        """Get appropriate log directory based on OS"""
        system = platform.system()
        
        if system == "Windows":
            log_base = os.path.join(os.environ.get('LOCALAPPDATA', tempfile.gettempdir()), 
                                   'AEPdowngrader', 'logs')
        elif system == "Darwin":  # macOS
            log_base = os.path.expanduser('~/Library/Logs/AEPdowngrader')
        else:  # Linux
            log_base = os.path.expanduser('~/.local/share/AEPdowngrader/logs')
        
        os.makedirs(log_base, exist_ok=True)
        return log_base
    
    def _log(self, level, message, extra_info=None):
        """Internal log method"""
        timestamp = datetime.now().isoformat()
        thread_id = threading.get_ident()
        
        log_entry = {
            "timestamp": timestamp,
            "level": level,
            "message": message,
            "thread_id": thread_id,
            "extra": extra_info or {},
        }
        
        # Format for output
        formatted = f"[{timestamp}] [{level}] [Thread-{thread_id}] {message}"
        if extra_info:
            formatted += f" | {json.dumps(extra_info)}"
        
        # Write to buffer
        with self._buffer_lock:
            self.log_buffer.write(formatted + "\n")
        
        # Write to file if open
        if self.log_file:
            self.log_file.write(formatted + "\n")
            self.log_file.flush()
        
        # Print to console if debug enabled
        if self.enabled:
            print(f"[DEBUG] {formatted}")
    
    def _log_summary(self):
        """Log session summary"""
        self._log(DebugLevel.INFO, "=== Session Summary ===")
        
        if self.start_time:
            duration = datetime.now() - self.start_time
            self._log(DebugLevel.INFO, f"Duration: {duration}")
        
        # Log file operations
        ops = self.fs_monitor.get_operations()
        self._log(DebugLevel.INFO, f"File operations: {len(ops)}")
        
        # Log network requests
        if self.network_monitor:
            reqs = self.network_monitor.get_requests()
            self._log(DebugLevel.INFO, f"Network requests: {len(reqs)}")
        
        # Log memory info
        mem = MemoryInfo.get_memory_info()
        self._log(DebugLevel.INFO, f"Final memory (RSS): {mem.get('rss_mb', 'N/A')} MB")
    
    def trace(self, message, extra_info=None):
        """Log trace level message"""
        if self.enabled:
            self._log(DebugLevel.TRACE, message, extra_info)
    
    def debug(self, message, extra_info=None):
        """Log debug level message"""
        if self.enabled:
            self._log(DebugLevel.DEBUG, message, extra_info)
    
    def info(self, message, extra_info=None):
        """Log info level message"""
        if self.enabled:
            self._log(DebugLevel.INFO, message, extra_info)
    
    def warning(self, message, extra_info=None):
        """Log warning level message"""
        if self.enabled:
            self._log(DebugLevel.WARNING, message, extra_info)
    
    def error(self, message, extra_info=None):
        """Log error level message"""
        if self.enabled:
            self._log(DebugLevel.ERROR, message, extra_info)
            # Also log stack trace
            stack = traceback.format_stack()
            self._log(DebugLevel.ERROR, f"Stack trace: {''.join(stack[-5:])}")
    
    def critical(self, message, extra_info=None):
        """Log critical level message"""
        if self.enabled:
            self._log(DebugLevel.CRITICAL, message, extra_info)
    
    def log_function_call(self, func_name, args=None, kwargs=None):
        """Log function call with parameters"""
        if self.enabled:
            extra = {
                "function": func_name,
                "args": str(args) if args else [],
                "kwargs": str(kwargs) if kwargs else {},
            }
            self._log(DebugLevel.TRACE, f"Calling: {func_name}", extra)
    
    def log_function_result(self, func_name, result=None, error=None):
        """Log function result"""
        if self.enabled:
            if error:
                self._log(DebugLevel.TRACE, f"Result: {func_name} - Error: {error}")
            else:
                self._log(DebugLevel.TRACE, f"Result: {func_name} - Success", {"result": str(result)[:200]})
    
    def log_memory(self, label=""):
        """Log current memory usage"""
        if self.enabled:
            mem = MemoryInfo.get_memory_info()
            self._log(DebugLevel.DEBUG, f"Memory ({label})", mem)
    
    def log_cpu(self, label=""):
        """Log current CPU usage"""
        if self.enabled:
            cpu = MemoryInfo.get_cpu_info()
            self._log(DebugLevel.DEBUG, f"CPU ({label})", cpu)
    
    def log_file_read(self, path, size=None):
        """Log file read operation"""
        if self.enabled:
            self.fs_monitor.log_read(path, size)
            self._log(DebugLevel.TRACE, f"Read file: {path}", {"size": size})
    
    def log_file_write(self, path, size=None):
        """Log file write operation"""
        if self.enabled:
            self.fs_monitor.log_write(path, size)
            self._log(DebugLevel.TRACE, f"Write file: {path}", {"size": size})
    
    def log_file_operation(self, operation, path, details=None):
        """Log generic file operation"""
        if self.enabled:
            self.fs_monitor.log_operation(operation, path, details)
            self._log(DebugLevel.TRACE, f"File operation: {operation} - {path}", details)
    
    def get_log_content(self):
        """Get all logged content as string"""
        with self._buffer_lock:
            return self.log_buffer.getvalue()
    
    def get_full_report(self):
        """Generate full debug report with system info"""
        report = StringIO()
        
        report.write("=" * 60 + "\n")
        report.write("AEP Downgrader - Debug Report\n")
        report.write("=" * 60 + "\n\n")
        
        # Session info
        report.write("SESSION INFORMATION\n")
        report.write("-" * 40 + "\n")
        report.write(f"Session ID: {self.session_id or 'N/A'}\n")
        report.write(f"Start Time: {self.start_time or 'N/A'}\n")
        if self.start_time:
            duration = datetime.now() - self.start_time
            report.write(f"Duration: {duration}\n")
        report.write("\n")
        
        # Platform info
        report.write("SYSTEM INFORMATION\n")
        report.write("-" * 40 + "\n")
        for key, value in self.platform_info.items():
            report.write(f"{key}: {value}\n")
        report.write("\n")
        
        # Memory info
        report.write("MEMORY INFORMATION\n")
        report.write("-" * 40 + "\n")
        mem = MemoryInfo.get_memory_info()
        for key, value in mem.items():
            report.write(f"{key}: {value}\n")
        report.write("\n")
        
        # CPU info
        report.write("CPU INFORMATION\n")
        report.write("-" * 40 + "\n")
        cpu = MemoryInfo.get_cpu_info()
        for key, value in cpu.items():
            report.write(f"{key}: {value}\n")
        report.write("\n")
        
        # File operations
        report.write("FILE OPERATIONS\n")
        report.write("-" * 40 + "\n")
        ops = self.fs_monitor.get_operations()
        report.write(f"Total operations: {len(ops)}\n\n")
        for op in ops[:50]:  # Limit to 50 entries
            report.write(f"  [{op['timestamp']}] {op['type']}: {op['path']}\n")
            if op.get('details'):
                for k, v in op['details'].items():
                    report.write(f"    {k}: {v}\n")
        if len(ops) > 50:
            report.write(f"  ... and {len(ops) - 50} more operations\n")
        report.write("\n")
        
        # Network requests
        if self.network_monitor:
            report.write("NETWORK REQUESTS\n")
            report.write("-" * 40 + "\n")
            reqs = self.network_monitor.get_requests()
            report.write(f"Total requests: {len(reqs)}\n\n")
            for req in reqs[:20]:
                report.write(f"  [{req['timestamp']}] {req['method']} {req['url']}\n")
                if req.get('response_code'):
                    report.write(f"    Response: {req['response_code']}\n")
                if req.get('error'):
                    report.write(f"    Error: {req['error']}\n")
            if len(reqs) > 20:
                report.write(f"  ... and {len(reqs) - 20} more requests\n")
            report.write("\n")
        
        # Log entries
        report.write("LOG ENTRIES\n")
        report.write("-" * 40 + "\n")
        report.write(self.get_log_content())
        
        return report.getvalue()
    
    def export_logs(self, file_path=None):
        """Export logs to file"""
        if file_path is None:
            # Generate default filename
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            log_dir = self._get_log_directory()
            file_path = os.path.join(log_dir, f"debug_export_{timestamp}.log")
        
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(self.get_full_report())
        
        return file_path
    
    def clear_logs(self):
        """Clear the log buffer"""
        with self._buffer_lock:
            self.log_buffer = StringIO()
        self.fs_monitor.clear()
        if self.network_monitor:
            self.network_monitor.clear()
    
    def is_enabled(self):
        """Check if debug mode is enabled"""
        return self.enabled


# Global debug logger instance
debug_logger = DebugLogger()
