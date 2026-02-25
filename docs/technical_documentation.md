# AEP Downgrader Technical Documentation

## Introduction

AEP Downgrader is a tool for converting Adobe After Effects (.aep) files from newer versions to older ones. The project uses deep analysis of RIFX file structures (a variant of the RIFF format with big-endian byte order) for safe modification of version-specific data.

## .aep File Structure

.aep files use the RIFX format (Resource Interchange File Format with big-endian byte order), which consists of:

- Signature: "RIFX"
- File size (4 bytes, big-endian)
- Format: "Egg!" (for .aep files)
- Sequence of chunks, each containing:
  - Chunk identifier (4 bytes)
  - Chunk data size (4 bytes, big-endian)
  - Chunk data
  - Padding byte if needed (if size is odd)

### Key Chunks

1. **head** - contains main After Effects version information
2. **nhed** - contains additional header data
3. **nnhd** - contains information about number of compositions and other elements
4. **LIST** - contains main project data

## File Version Detection

### Research Methods

To determine differences between versions, we conducted the following research:

1. **Comparative analysis** of files from different AE versions
2. **Byte-level analysis** of header chunk structures
3. **Identification of key bytes** that distinguish versions

### Key Bytes for Version Detection

Based on analysis of files from various AE versions, we determined that key differences are located in the `head` chunk at the following positions (relative to the start of chunk data):

- Position 1: Main version identifier
- Position 3: Additional identifier
- Position 4: Project type
- Position 5: Additional information
- Position 6: Checksum/identifier
- Position 7: Additional flag

### Known Version Signatures

| Version | Bytes [1,3,4,5,6,7] | Hex |
|---------|---------------------|-----|
| AE 25.x | [0x60, 0x01, 0x0f, 0x08, 0x86, 0x44] | [96, 1, 15, 8, 134, 68] |
| AE 24.x | [0x5f, 0x05, 0x0f, 0x02, 0x86, 0x34] | [95, 5, 15, 2, 134, 52] |
| AE 23.x | [0x5e, 0x09, 0x0b, 0x3b, 0x06, 0x37] | [94, 9, 11, 59, 6, 55] |
| AE 22.x | [0x5d, 0x2b, 0x0b, 0x33, 0x06, 0x3b] | [93, 43, 11, 51, 6, 59] |

Note that the first byte decreases with increasing version: 0x60 (25) → 0x5f (24) → 0x5e (23) → 0x5d (22).

## Conversion Process

### Transformation Algorithm

1. **File Reading**: Open the .aep file in binary mode
2. **Header Analysis**: Extract data from the `head` chunk
3. **Version Detection**: Compare with known signatures
4. **Signature Conversion**: Replace bytes with those corresponding to the target version
5. **Result Writing**: Save the modified file

### Byte Positioning

For precise byte modification in the file, we use the following offsets:

- Start of `head` chunk data: 32 bytes from the beginning of the file
- Positions to modify: [1, 3, 4, 5, 6, 7] relative to the start of chunk data
- Final file offsets: 32 + [1, 3, 4, 5, 6, 7] = [33, 35, 36, 37, 38, 39]

## Research History

### Stage 1: File Structure Analysis

Initially, we determined that .aep files use the RIFX format rather than standard RIFF. This allowed us to correctly interpret chunk sizes and data.

### Stage 2: Comparison of Files from Different Versions

We compared files created in AE 22.x, 23.x, 24.x, and 25.x to find structural differences. The key discovery was made during analysis of header chunks.

### Stage 3: Identification of Key Bytes

After detailed analysis, we determined that 6 specific bytes in the `head` chunk uniquely identify the After Effects version.

### Stage 4: Conversion Testing

We tested various transformation combinations and verified whether the results opened correctly in target AE versions.

### Stage 5: Discovery of AE 22.x Issues

During testing of conversion to AE 22.x, structural differences were discovered that caused errors when opening files. Based on these observations, we decided to remove AE 22.x support due to potential compatibility issues.

## Resources Used

This project is partially based on research conducted in the [aep-parser](https://github.com/uwe-mayer/aep-parser) repository, which provided valuable insights into .aep file structure.

## Limitations

1. **Supported Versions**: Currently only conversions to AE 24.x and AE 23.x are supported
2. **Structural Differences**: Some complex projects may contain version-specific structural features
3. **Plugins**: Effects and plugins unavailable in the target version may cause issues

## Future Improvements

1. **Extended Support**: Ability to handle additional structural differences between versions
2. **Integrity Checks**: Addition of checks to ensure correctness of converted files
3. **Additional Version Support**: Expansion of support after further research
4. **Integrated Debug Mode**: Full debug functionality will be included in the application (currently requires separate `psutil` dependency)