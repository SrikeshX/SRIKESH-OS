from PIL import Image

img = Image.open("/mnt/c/Users/shres/.gemini/antigravity/brain/43e9f22a-f7d1-4d29-a088-e60efad3b868/video_frames/shot_014.png")
w, h = img.size
# On screen, crop the window area
# In shot_014, the black box is roughly y: 40% to 70%, x: 15% to 85%
crop = img.crop((int(w*0.18), int(h*0.35), int(w*0.82), int(h*0.75)))
crop.save("/mnt/c/Users/shres/.gemini/antigravity/brain/43e9f22a-f7d1-4d29-a088-e60efad3b868/video_frames/window_crop_014.png")

# Also crop shot_011
img11 = Image.open("/mnt/c/Users/shres/.gemini/antigravity/brain/43e9f22a-f7d1-4d29-a088-e60efad3b868/video_frames/shot_011.png")
crop11 = img11.crop((int(w*0.15), int(h*0.08), int(w*0.85), int(h*0.58)))
crop11.save("/mnt/c/Users/shres/.gemini/antigravity/brain/43e9f22a-f7d1-4d29-a088-e60efad3b868/video_frames/window_crop_011.png")

# Also crop the dock at the bottom in shot_014
dock_crop = img.crop((int(w*0.3), int(h*0.88), int(w*0.7), int(h*0.98)))
dock_crop.save("/mnt/c/Users/shres/.gemini/antigravity/brain/43e9f22a-f7d1-4d29-a088-e60efad3b868/video_frames/dock_crop_014.png")
print("Saved crops.")
