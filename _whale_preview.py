import time, sys
width = 40; direction = 1; pos = 0; whales = ['🐋', '🐳']; frame = 0
for _ in range(80):
    bar = [' '] * width
    bar[pos] = whales[frame % 2]
    sys.stdout.write('\r[' + ''.join(bar) + ']')
    sys.stdout.flush()
    pos += direction; frame += 1
    if pos <= 0 or pos >= width - 1: direction *= -1
    time.sleep(0.05)
print()
