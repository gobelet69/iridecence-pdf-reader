import re

with open('src/main.rs', 'r') as f:
    lines = f.readlines()

for i, line in enumerate(lines):
    if re.match(r'^ *(pub )?(class|struct|impl|fn) ', line):
        print(f"{i+1}: {line.strip()}")
