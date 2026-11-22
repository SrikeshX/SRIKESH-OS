from PIL import Image
import glob

# Check if Pharaoh or Anubis is in each frame
# Anubis has a prominent black wolf head with yellow eyes.
# Pharaoh has gold headpiece (nemes) with blue stripes.
for f in sorted(glob.glob("/mnt/c/Users/shres/.gemini/antigravity/brain/43e9f22a-f7d1-4d29-a088-e60efad3b868/video_frames/shot_*.png")):
    img = Image.open(f)
    w, h = img.size
    # crop near upper center
    crop = img.crop((int(w*0.4), int(h*0.2), int(w*0.6), int(h*0.4)))
    rgb = crop.convert("RGB")
    # count very dark pixels (wolf fur) vs golden pixels
    dark = 0
    gold = 0
    for r, g, b in rgb.getdata():
        if r < 40 and g < 40 and b < 50:
            dark += 1
        elif r > 150 and g > 120 and b < 80:
            gold += 1
    total = crop.width * crop.height
    print(f"{f[-12:]}: dark={dark/total:.2f}, gold={gold/total:.2f}")
