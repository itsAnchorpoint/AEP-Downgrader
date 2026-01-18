#!/usr/bin/env python3
"""
Deep analysis tool to compare AE 23.x and AE 22.x files
"""
import os
import struct


def read_chunks_detailed(file_path):
    """
    Read all chunks from the RIFX file with detailed information
    """
    chunks = []
    
    with open(file_path, 'rb') as f:
        # Read RIFX header
        signature = f.read(4)
        if signature not in [b'RIFF', b'RIFX']:
            raise ValueError(f"File is not in RIFF/RIFX format. Got: {signature}")
        
        # File size (big-endian)
        size_bytes = f.read(4)
        file_size = struct.unpack('>I', size_bytes)[0]
        
        # Format type
        format_type = f.read(4)
        
        print(f"File: {file_path}")
        print(f"Signature: {signature}, Size: {file_size}, Format: {format_type}")
        
        # Read all chunks
        while f.tell() < os.path.getsize(file_path):
            chunk_start_pos = f.tell()
            
            # Read chunk header (ID + Size)
            chunk_header = f.read(8)
            if len(chunk_header) < 8:
                break  # End of file
            
            chunk_id = chunk_header[:4]
            chunk_size = struct.unpack('>I', chunk_header[4:8])[0]  # Big-endian
            
            # Read chunk data
            chunk_data = f.read(chunk_size)
            
            # Store chunk info
            chunk_info = {
                'id': chunk_id,
                'id_str': chunk_id.decode('ascii', errors='ignore'),
                'size': chunk_size,
                'start_pos': chunk_start_pos,
                'data': chunk_data,
                'preview': chunk_data[:50]  # First 50 bytes for preview
            }
            
            chunks.append(chunk_info)
            
            # Skip padding byte if chunk size is odd
            if chunk_size % 2:
                f.read(1)
    
    return chunks


def compare_files_detailed(file1_path, file2_path):
    """
    Compare two .aep files in detail to identify all differences
    """
    print("="*80)
    print(f"COMPARING FILES IN DETAIL:")
    print(f"File 1: {os.path.basename(file1_path)}")
    print(f"File 2: {os.path.basename(file2_path)}")
    print("="*80)
    
    # Read chunks from both files
    chunks1 = read_chunks_detailed(file1_path)
    chunks2 = read_chunks_detailed(file2_path)
    
    print(f"\n{os.path.basename(file1_path)} has {len(chunks1)} chunks")
    print(f"{os.path.basename(file2_path)} has {len(chunks2)} chunks")
    
    # Compare each chunk
    max_chunks = max(len(chunks1), len(chunks2))
    differences = []
    
    for i in range(max_chunks):
        chunk1 = chunks1[i] if i < len(chunks1) else None
        chunk2 = chunks2[i] if i < len(chunks2) else None
        
        if chunk1 is None:
            print(f"Chunk #{i+1}: Only in file 2 - ID='{chunk2['id_str']}', Size={chunk2['size']}")
            differences.append(f"Extra chunk in file 2: #{i+1} '{chunk2['id_str']}'")
        elif chunk2 is None:
            print(f"Chunk #{i+1}: Only in file 1 - ID='{chunk1['id_str']}', Size={chunk1['size']}")
            differences.append(f"Extra chunk in file 1: #{i+1} '{chunk1['id_str']}'")
        elif chunk1['id_str'] != chunk2['id_str']:
            print(f"Chunk #{i+1}: Different IDs - File1='{chunk1['id_str']}', File2='{chunk2['id_str']}'")
            differences.append(f"Different chunk IDs at #{i+1}: '{chunk1['id_str']}' vs '{chunk2['id_str']}'")
        elif chunk1['size'] != chunk2['size']:
            print(f"Chunk #{i+1} ('{chunk1['id_str']}'): Different sizes - File1={chunk1['size']}, File2={chunk2['size']}")
            differences.append(f"Different sizes for '{chunk1['id_str']}' at #{i+1}: {chunk1['size']} vs {chunk2['size']}")
        else:
            # Same ID and size, check content
            if chunk1['data'] != chunk2['data']:
                print(f"Chunk #{i+1} ('{chunk1['id_str']}'): Same size but different content")
                differences.append(f"Different content for '{chunk1['id_str']}' at #{i+1}")
                
                # Show first few differing bytes
                diff_positions = []
                for j, (b1, b2) in enumerate(zip(chunk1['data'], chunk2['data'])):
                    if b1 != b2:
                        diff_positions.append(j)
                        if len(diff_positions) >= 10:  # Show first 10 differences
                            break
                print(f"  First different byte positions: {diff_positions}")
    
    print(f"\nTotal differences found: {len(differences)}")
    if differences:
        print("Differences:")
        for diff in differences[:20]:  # Show first 20 differences
            print(f"  - {diff}")
        if len(differences) > 20:
            print(f"  ... and {len(differences) - 20} more differences")
    
    # Special analysis for LIST chunks which might contain size differences
    list_chunks1 = [c for c in chunks1 if c['id_str'] == 'LIST']
    list_chunks2 = [c for c in chunks2 if c['id_str'] == 'LIST']
    
    print(f"\nDetailed LIST chunks analysis:")
    print(f"  File 1 has {len(list_chunks1)} LIST chunks")
    print(f"  File 2 has {len(list_chunks2)} LIST chunks")
    
    for i, (lc1, lc2) in enumerate(zip(list_chunks1, list_chunks2)):
        if lc1['size'] != lc2['size']:
            print(f"  LIST chunk #{i+1}: File1={lc1['size']}, File2={lc2['size']}, Diff={lc1['size'] - lc2['size']}")
    
    return chunks1, chunks2, differences


def main():
    # Compare AE 23.x and AE 22.x files
    ae23_file = "examples/dev_aeps/TEST23.x.aep"
    ae22_file = "examples/dev_aeps/TEST22.x.aep"
    
    if not os.path.exists(ae23_file):
        print(f"File not found: {ae23_file}")
        ae23_file = "../examples/dev_aeps/TEST23.x.aep"  # Try relative path
    
    if not os.path.exists(ae22_file):
        print(f"File not found: {ae22_file}")
        ae22_file = "../examples/dev_aeps/TEST22.x.aep"  # Try relative path
    
    if os.path.exists(ae23_file) and os.path.exists(ae22_file):
        compare_files_detailed(ae23_file, ae22_file)
    else:
        print(f"Could not find both files to compare.")
        print(f"Looked for: {ae23_file} and {ae22_file}")


if __name__ == "__main__":
    main()