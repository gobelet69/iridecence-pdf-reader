import re

with open('src/main.rs', 'r') as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if 'Home' in line or 'Favorites' in line or 'Recent' in line:
        print(f"{i+1}: {line.strip()}")
