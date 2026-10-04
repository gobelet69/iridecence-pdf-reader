with open('src/main.rs', 'r') as f:
    lines = f.readlines()

depth = 0
for i, line in enumerate(lines):
    # Ignore braces in strings/comments for this basic heuristic (we probably don't have weird ones)
    line_clean = line.split('//')[0]
    opens = line_clean.count('{')
    closes = line_clean.count('}')
    depth += opens - closes
    
    if "fn " in line and depth == 1: # We are inside AppDelegate impl
        print(f"Function starts at line {i+1}")
    if depth == 1 and line.strip() == "}":
        print(f"Function ends before line {i+1}")

