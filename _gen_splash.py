from PIL import Image, ImageDraw, ImageFont
import sys, os

# Read version from Package.appxmanifest
manifest = "src/NotAlterra.UI/Package.appxmanifest"
ver = "v0.5.0"
for line in open(manifest, encoding="utf-8"):
    if 'Version="' in line:
        v = line.split('Version="')[1].split('"')[0]
        ver = "v" + v
        break

# Allow override from command line
if len(sys.argv) > 1:
    ver = sys.argv[1]

path = "src/NotAlterra.UI/Assets/splash.png"
font_path = "C:/Windows/Fonts/SEGOEUI.TTF"

img = Image.open(path).convert("RGBA")
w, h = img.size
draw = ImageDraw.Draw(img)

fl = ImageFont.truetype(font_path, 10)

bb = draw.textbbox((0, 0), ver, font=fl)
x = w - bb[2] - 16
y = h - bb[3] - 16
draw.text((x, y), ver, font=fl, fill=(255, 255, 255, 160))

img.save(path)
print(f"splash version: {ver}")
