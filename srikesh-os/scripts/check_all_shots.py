from PIL import Image
import os, glob

files = sorted(glob.glob("/mnt/c/Users/shres/.gemini/antigravity/brain/43e9f22a-f7d1-4d29-a088-e60efad3b868/video_frames/shot_*.png"))
for f in files:
    img = Image.open(f)
    w, h = img.size
    c_pixel = img.getpixel((w//2, h//2))
    # check if center is dark (window)
    print(f"{os.path.basename(f)}: center={c_pixel}")
