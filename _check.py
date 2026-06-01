import re
missing = []
for fname in ['src/main.rs','src/tui.rs','src/gvas.rs','src/ops.rs','src/discovery.rs','src/guard.rs','src/config.rs']:
    lines = open(fname).readlines()
    for i, line in enumerate(lines):
        if re.match(r'^\s*fn\s+\w+', line):
            prev = lines[i-1].strip() if i > 0 else ''
            if not prev.startswith('///') and not prev.startswith('//'):
                name = line.strip().split('(')[0].replace('fn ','')
                missing.append((fname, i+1, name))
for f,l,n in missing:
    print(f'{f}:{l}: {n}')
