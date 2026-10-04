with open('src/main.rs', 'r') as f:
    lines = f.readlines()
for i, line in enumerate(lines[650:680]):
    print(f"{i+650}: {line.rstrip()}")
