#!/usr/bin/env python3
"""
AEP Downgrader - Setup Script

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
from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="ae-downgrader",
    version="1.0.0",
    author="Your Name",
    author_email="your.email@example.com",
    description="A utility to downgrade Adobe After Effects project files from newer versions to older ones",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/yourusername/ae-downgrader",
    packages=find_packages(),
    install_requires=[
        'PyQt5>=5.15.0',
        'psutil>=5.9.0',
    ],
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: GNU General Public License v3 (GPLv3)",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
    ],
    python_requires=">=3.9",
    entry_points={
        'console_scripts': [
            'ae-downgrader=final_downgrader:main',
        ],
    },
)