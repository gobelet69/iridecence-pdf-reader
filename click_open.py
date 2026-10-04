import time
import subprocess
import pyautogui

print("Waiting for app to launch...")
time.sleep(3)
print("Clicking Open Folder...")
# Assuming the window is in the center, or we can just send keystrokes!
# Wait, "Open Folder..." doesn't have a shortcut.
# We can use osascript to click it!
