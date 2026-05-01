import os
import re

total_lines = 0
files_processed = 0

for root, dirs, files in os.walk('src'):
    for file in files:
        if file.endswith('.rs'):
            filepath = os.path.join(root, file)
            files_processed += 1
            try:
                with open(filepath, 'r', encoding='utf-8') as f:
                    content = f.read()
                    lines = content.splitlines()
                    total_lines += len(lines)
                    
                    # Find arrays like [ ... ] with many elements
                    for match in re.finditer(r'const\s+\w+\s*:[^=]+=\s*\[([^\]]{500,})\]', content):
                        print(f'Large array found in {filepath}')
                    
                    # Find long strings
                    for match in re.finditer(r'\"([^\"]{500,})\"', content):
                        print(f'Long string found in {filepath}')
            except:
                pass

print(f'Total lines of Rust code: {total_lines}')
print(f'Total files: {files_processed}')
