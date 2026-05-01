import sys

files_to_fix = [
    'src/events.rs', 'src/invoice.rs', 'src/invoice_search.rs', 
    'src/storage.rs', 'src/types.rs', 'src/verification.rs', 'src/lib.rs'
]

lines_to_fix = {}
for line in open('parsed_spans.txt', 'r'):
    parts = line.strip().split(':')
    if len(parts) == 4 and parts[0] == 'unused_imports':
        file = parts[1].replace('\\', '/')
        if any(file.endswith(f) for f in files_to_fix):
            lines_to_fix.setdefault(file, set()).add(int(parts[2]))

for file, lines in lines_to_fix.items():
    try:
        with open(file, 'r', encoding='utf-8') as f:
            content = f.read().split('\n')
        
        for lno in sorted(lines, reverse=True):
            idx = lno - 1
            if 0 <= idx < len(content):
                # We can comment out the import line
                if not content[idx].strip().startswith('//'):
                    content[idx] = '// ' + content[idx]
        
        with open(file, 'w', encoding='utf-8') as f:
            f.write('\n'.join(content))
        print(f"Fixed {len(lines)} imports in {file}")
    except Exception as e:
        print(f"Error processing {file}: {e}")
