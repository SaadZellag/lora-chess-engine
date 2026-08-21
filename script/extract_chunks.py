import argparse
import struct


def extract_chunks(input_file, num_chunks, output_file):
    """
    Extract the first N chunks from a stockfish binpack file and write to output file.
    
    Chunk structure:
    - First 4 bytes: 'BINP' (magic number)
    - Next 4 bytes: chunk size in bytes (little-endian int)
    - Next N bytes: chunk data
    """
    with open(input_file, 'rb') as infile, open(output_file, 'wb') as outfile:
        chunks_extracted = 0
        
        while chunks_extracted < num_chunks:
            # Read magic number (4 bytes)
            magic = infile.read(4)
            if not magic or len(magic) < 4:
                print(f"Warning: End of file reached after {chunks_extracted} chunks")
                break
            
            if magic != b'BINP':
                print(f"Error: Expected 'BINP' magic number, got {magic}")
                break
            
            # Read chunk size (next 4 bytes, little-endian)
            chunk_size_bytes = infile.read(4)
            if len(chunk_size_bytes) < 4:
                print(f"Error: Could not read chunk size at chunk {chunks_extracted}")
                break
            
            chunk_size = struct.unpack('<I', chunk_size_bytes)[0]
            
            # Read chunk data
            chunk_data = infile.read(chunk_size)
            if len(chunk_data) < chunk_size:
                print(f"Error: Expected {chunk_size} bytes, got {len(chunk_data)} at chunk {chunks_extracted}")
                break
            
            # Write to output file (magic + size + data)
            outfile.write(magic)
            outfile.write(chunk_size_bytes)
            outfile.write(chunk_data)
            
            chunks_extracted += 1
            print(f"Extracted chunk {chunks_extracted}/{num_chunks} (size: {chunk_size} bytes)")
        
        print(f"Successfully extracted {chunks_extracted} chunks to {output_file}")


def main():
    parser = argparse.ArgumentParser(
        description='Extract N chunks from a stockfish binpack file'
    )
    parser.add_argument(
        '--input-file',
        required=True,
        help='Path to the input binpack file'
    )
    parser.add_argument(
        '--num-chunks',
        type=int,
        required=True,
        help='Number of chunks to extract'
    )
    parser.add_argument(
        '--output-file',
        required=True,
        help='Path to the output file'
    )
    
    args = parser.parse_args()
    
    try:
        extract_chunks(args.input_file, args.num_chunks, args.output_file)
    except FileNotFoundError as e:
        print(f"Error: {e}")
        return 1
    except Exception as e:
        print(f"Error: {e}")
        return 1
    
    return 0


if __name__ == '__main__':
    main()
