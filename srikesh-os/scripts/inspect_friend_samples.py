import os
from PIL import Image

out_dir = r"C:\Users\shres\.gemini\antigravity\brain\43e9f22a-f7d1-4d29-a088-e60efad3b868\friend_crops"

# Let's crop specific regions from sample_015.png, sample_025.png, sample_045.png, sample_065.png
# Top-left, Top-right, Bottom-left, Bottom-right, Center
for k in [1, 8, 15, 25, 45, 65]:
    im = Image.open(os.path.join(out_dir, f"sample_{k:03d}.png"))
    w, h = im.size
    im.crop((0, 0, 200, 40)).save(os.path.join(out_dir, f"top_left_{k:03d}.png"))
    im.crop((w-200, 0, w, 40)).save(os.path.join(out_dir, f"top_right_{k:03d}.png"))
    im.crop((0, h-40, 200, h)).save(os.path.join(out_dir, f"bottom_left_{k:03d}.png"))
    im.crop((w-200, h-40, w, h)).save(os.path.join(out_dir, f"bottom_right_{k:03d}.png"))
    im.crop((100, 60, w-100, h-60)).save(os.path.join(out_dir, f"center_{k:03d}.png"))

print("Cropped regions successfully")
