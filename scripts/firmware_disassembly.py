"""Wrap a read-only ARM image for LLVM decoding. Never executes target code."""
import struct
import subprocess
import tempfile
from pathlib import Path


def disassemble_arm(image, base, ranges, thumb=True, label='<user.bin ELF wrapper>'):
    """Return bounded Thumb or ARM listings using the installed Xcode LLVM tool."""
    names = b'\0.text\0.shstrtab\0'
    text_offset = 52
    names_offset = text_offset + len(image)
    table_offset = (names_offset + len(names) + 3) & ~3
    header = struct.pack(
        '<16sHHIIIIIHHHHHH', b'\x7fELF\x01\x01\x01' + bytes(9),
        2, 40, 1, base | 1, 0, table_offset, 0x05000000,
        52, 0, 0, 40, 3, 2,
    )
    sections = bytes(40) + struct.pack(
        '<10I', 1, 1, 6, base, text_offset, len(image), 0, 0, 4, 0,
    ) + struct.pack('<10I', 7, 3, 0, 0, names_offset, len(names), 0, 0, 1, 0)
    wrapped = (header + image + names).ljust(table_offset, b'\0') + sections
    outputs = []
    with tempfile.TemporaryDirectory(prefix='ta1278-static-') as directory:
        elf = Path(directory) / 'user-analysis.elf'
        elf.write_bytes(wrapped)
        for start, end in ranges:
            result = subprocess.run([
                'xcrun', 'llvm-objdump', '-d', '--triple='+('thumbv7a' if thumb else 'armv7a')+'-none-eabi',
                '--mattr=+hwdiv', f'--start-address={start}',
                f'--stop-address={end}', str(elf),
            ], check=True, text=True, capture_output=True)
            outputs.append(result.stdout.replace(str(elf), label))
    return '\n'.join(outputs)
