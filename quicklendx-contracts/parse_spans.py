import json
spans = []
for file in ['check.json', 'check_errs.json']:
    try:
        for line in open(file, 'r', encoding='utf-8'):
            try:
                data = json.loads(line.strip())
                if data.get('reason') == 'compiler-message' and data.get('message'):
                    msg = data['message']
                    if msg.get('code'):
                        code = msg['code']['code']
                        if code in ('dead_code', 'unused_imports', 'deprecated'):
                            for s in msg.get('spans', []):
                                if s.get('is_primary'):
                                    spans.append(f"{code}:{s['file_name']}:{s['line_start']}:{s['line_end']}")
            except Exception: pass
    except Exception: pass
print(len(spans))
open('parsed_spans.txt', 'w', encoding='utf-8').write('\n'.join(spans))
