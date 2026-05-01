import json
import os

warnings = []
if os.path.exists('check.json'):
    with open('check.json', 'r') as f:
        for line in f:
            try:
                data = json.loads(line)
                if data.get('reason') == 'compiler-message':
                    msg = data.get('message')
                    if msg.get('level') == 'warning':
                        code_obj = msg.get('code')
                        code = code_obj.get('code') if code_obj else 'unknown'
                        spans = msg.get('spans', [])
                        for span in spans:
                            if span.get('is_primary'):
                                warnings.append(f"{code}:{span.get('file_name')}:{span.get('line_start')}:{span.get('column_start')}")
            except:
                pass

with open('warnings_new.txt', 'w') as f:
    for w in sorted(list(set(warnings))):
        f.write(w + '\n')
print(f'Found {len(warnings)} unique warnings')
