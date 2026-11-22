from PIL import Image
import os, glob

files = sorted(glob.glob("/mnt/c/Users/shres/.gemini/antigravity/brain/43e9f22a-f7d1-4d29-a088-e60efad3b868/video_frames/shot_*.png"))
for f in files:
    img = Image.open(f).convert("L")
    w, h = img.size
    center = img.crop((int(w*0.2), int(h*0.2), int(w*0.8), int(h*0.8)))
    bottom = img.crop((0, int(h*0.7), w, h))
    c_mean = sum(center.getdata()) / (center.width * center.height)
    b_mean = sum(bottom.getdata()) / (bottom.width * bottom.height)
    print(f"{os.path.basename(f)}: center={c_mean:.1f}, bottom={b_mean:.1f}")
