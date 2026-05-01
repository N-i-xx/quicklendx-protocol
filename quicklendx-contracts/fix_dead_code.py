import sys
import re
from collections import defaultdict

def main():
    # Parse parsed_spans.txt
    dead_code = defaultdict(list)
    try:
        with open("parsed_spans.txt", "r", encoding="utf-8") as f:
            for line in f:
                if line.startswith("dead_code:"):
                    parts = line.strip().split(":")
                    if len(parts) >= 4:
                        filepath = parts[1]
                        line_start = int(parts[2])
                        dead_code[filepath].append(line_start)
    except Exception as e:
        print(f"Error reading parsed_spans.txt: {e}")
        return

    for filepath, lines in dead_code.items():
        try:
            with open(filepath, "r", encoding="utf-8") as f:
                file_lines = f.readlines()
        except FileNotFoundError:
            continue
        
        # Sort lines descending so insertions don't change line numbers of preceding warnings
        lines = sorted(list(set(lines)), reverse=True)
        
        changed = False
        for l in lines:
            idx = l - 1  # 0-indexed
            if idx < len(file_lines):
                # Don't add if already there
                if idx > 0 and "#[allow(dead_code)]" in file_lines[idx - 1]:
                    continue
                # Match indentation
                indent = len(file_lines[idx]) - len(file_lines[idx].lstrip())
                file_lines.insert(idx, " " * indent + "#[allow(dead_code)]\n")
                changed = True
        
        if changed:
            with open(filepath, "w", encoding="utf-8") as f:
                f.writelines(file_lines)
            print(f"Fixed dead_code in {filepath}")

if __name__ == "__main__":
    main()
