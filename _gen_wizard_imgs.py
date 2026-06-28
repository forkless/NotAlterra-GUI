from PIL import Image

i = Image.open("src/NotAlterra.UI/Assets/na.png").convert("RGBA")

# Small logo for wizard pages (55x55)
s = i.resize((55, 55))
# Composite onto orange background
bg_small = Image.new("RGB", (55, 55), (255, 255, 255))
bg_small.paste(s, (0, 0), s)
bg_small.save("src/NotAlterra.UI/Assets/setup_logo.bmp")

# Large background for Welcome/Finished pages (164x314)
logo = i.resize((80, 80))
bg = Image.new("RGB", (164, 314), (255, 255, 255))
x = (164 - 80) // 2
y = (314 - 80) // 2 - 30
bg.paste(logo, (x, y), logo)
bg.save("src/NotAlterra.UI/Assets/setup_bg.bmp")

print("done")
