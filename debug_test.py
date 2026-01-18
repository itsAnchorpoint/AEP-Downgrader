#!/usr/bin/env python3
"""
Debug version to isolate the error
"""
import struct

def test_struct_unpack():
    """Test struct operations that might cause the error"""
    try:
        # Simulate the problematic operation
        test_data = b'\x00\x01\x02\x03'  # Valid 4-byte sequence
        result = struct.unpack('>I', test_data)[0]
        print(f"Valid unpack result: {result}")
        
        # Test with None (this should cause the error)
        try:
            none_data = None
            if none_data is not None:
                result = struct.unpack('>I', none_data)[0]
            else:
                print("Data is None, skipping unpack")
        except TypeError as e:
            print(f"Expected error with None: {e}")
            
    except Exception as e:
        print(f"Error in test_struct_unpack: {e}")
        import traceback
        traceback.print_exc()

def simulate_chunk_parsing():
    """Simulate the chunk parsing that might be causing the issue"""
    try:
        with open('examples/dev_aeps/TEST24.x.aep', 'rb') as f:
            content = f.read()
        
        # Look for LIST chunks similar to what was in the problematic code
        search_start = 140
        while search_start < len(content) - 8:
            if content[search_start:search_start+4] == b'LIST':
                print(f"Found LIST at offset {search_start}")
                if search_start + 8 <= len(content):
                    size_bytes = content[search_start+4:search_start+8]
                    print(f"Size bytes: {size_bytes}")
                    if size_bytes is not None and len(size_bytes) == 4:
                        current_size = struct.unpack('>I', size_bytes)[0]
                        print(f"Chunk size: {current_size}")
                    else:
                        print(f"Invalid size_bytes: {size_bytes}")
                    
                    # Move past this chunk
                    chunk_end = search_start + 8 + current_size
                    if current_size % 2:  # Odd sizes have padding byte
                        chunk_end += 1
                    search_start = chunk_end
                else:
                    break
            else:
                search_start += 1
                
    except Exception as e:
        print(f"Error in simulate_chunk_parsing: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    print("Testing struct operations...")
    test_struct_unpack()
    print("\nTesting chunk parsing...")
    simulate_chunk_parsing()
    print("\nDone.")