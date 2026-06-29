#!/usr/bin/env python3
"""Generate Inno Setup wizard images from AppIcon256.png"""
from PIL import Image

src = Image.open("src/NotAlterra.UI/Assets/AppIcon256.png").convert("RGBA")

# Small logo (55x55) for non-Welcome pages — 10px white padding all around
s = src.resize((35, 35))
bg = Image.new("RGB", (55, 55), (255, 255, 255))
mask = s.split()[3]  # alpha channel
bg.paste(s, (10, 10), mask)
bg.save("src/NotAlterra.UI/Assets/setup_logo.bmp")

# Large background (164x314) for Welcome/Finished pages
s2 = src.resize((80, 80))
bg2 = Image.new("RGB", (164, 314), (255, 255, 255))
mask2 = s2.split()[3]
x = (164 - 80) // 2
y = (314 - 80) // 2 - 30
bg2.paste(s2, (x, y), mask2)
bg2.save("src/NotAlterra.UI/Assets/setup_bg.bmp")
