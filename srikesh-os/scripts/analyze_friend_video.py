import os
from PIL import Image

frames_dir = r"C:\Users\shres\.gemini\antigravity\brain\43e9f22a-f7d1-4d29-a088-e60efad3b868\friend_video_frames"
out_dir = r"C:\Users\shres\.gemini\antigravity\brain\43e9f22a-f7d1-4d29-a088-e60efad3b868\friend_crops"
os.makedirs(out_dir, exist_ok=True)

# Save snapshots of key frames for inspection
for k in [1, 8, 15, 25, 35, 45, 55, 65, 75, 85, 95]:
    p = os.path.join(frames_dir, f"frame_{k:03d}.png")
    im = Image.open(p)
    im.save(os.path.join(out_dir, f"sample_{k:03d}.png"))

print("Saved sample frames to friend_crops")
