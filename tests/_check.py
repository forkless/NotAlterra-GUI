import re, sys

files = ['src/main.rs','src/tui.rs','src/gvas.rs','src/ops.rs','src/discovery.rs','src/guard.rs','src/config.rs','src/main.rs']
missing = []

for fname in files:
    lines = [l.rstrip() for l in open(fname).readlines()]
    for i, line in enumerate(lines):
        m = re.match(r'^\s*fn\s+(\w+)', line)
        if not m:
            continue
        name = m.group(1)
        # skip impl methods and test functions
        if name in ('new', 'default') or name.startswith('test_') or re.search(r'test_derive|test_corruption|dump_all', name):
            continue
        # skip functions with _ prefix (inactive guards)
        if name.startswith('_'):
            continue
        # check previous non-blank, non-attribute line for doc comment
        prev_idx = i - 1
        while prev_idx >= 0 and not lines[prev_idx].strip():
            prev_idx -= 1
        if prev_idx >= 0:
            stripped = lines[prev_idx].strip()
            # skip attributes and closing braces
            if stripped.startswith('#[') or stripped.startswith(']') or stripped == '}':
                prev_idx -= 1
                while prev_idx >= 0 and not lines[prev_idx].strip():
                    prev_idx -= 1
            if prev_idx >= 0:
                stripped = lines[prev_idx].strip()
                if not stripped.startswith('///') and not stripped.startswith('//'):
                    missing.append((fname, i + 1, name))
        else:
            missing.append((fname, i + 1, name))

if missing:
    print(f'{len(missing)} functions missing doc comments:')
    for f, l, n in sorted(missing):
        print(f'  {f}:{l}  {n}')
    sys.exit(1)
else:
    print('All functions documented.')
