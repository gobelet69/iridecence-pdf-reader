with open('src/main.rs', 'r') as f:
    lines = f.readlines()

start = False
depth = 0
for i, line in enumerate(lines):
    if "fn application_did_finish_launching" in line:
        start = True
    
    if start:
        line_clean = line.split('//')[0]
        o = line_clean.count('{')
        c = line_clean.count('}')
        depth += o - c
        print(f"{i+1:4}: depth {depth:2} | {line.rstrip()}")
        
        if depth == 0:
            break
