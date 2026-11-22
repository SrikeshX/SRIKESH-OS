import os
import cv2
import numpy as np

video_path = "/mnt/c/Users/shres/.gemini/antigravity/brain/43e9f22a-f7d1-4d29-a088-e60efad3b868/.user_uploaded/media_1795254216619_9635b187.mp4"
out_dir = "/mnt/c/Users/shres/.gemini/antigravity/brain/43e9f22a-f7d1-4d29-a088-e60efad3b868/video_frames"
os.makedirs(out_dir, exist_ok=True)

cap = cv2.VideoCapture(video_path)
total_frames = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
fps = cap.get(cv2.CAP_PROP_FPS)
print(f"Total frames: {total_frames}, FPS: {fps:.2f}, Duration: {total_frames/fps:.2f}s")

# Extract at regular intervals: every 0.5 seconds
step = int(fps * 0.5)
if step < 1:
    step = 1

frame_idx = 0
saved = 0
while True:
    ret, frame = cap.read()
    if not ret:
        break
    if frame_idx % step == 0:
        sec = frame_idx / fps
        out_name = f"{out_dir}/shot_{saved:02d}_{sec:04.1f}s.png"
        cv2.imwrite(out_name, frame)
        saved += 1
    frame_idx += 1

cap.release()
print(f"Extracted {saved} frames.")
