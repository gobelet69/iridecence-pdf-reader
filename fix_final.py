with open('src/main.rs', 'r') as f:
    lines = f.readlines()

lines.insert(668, "            }\n")

with open('src/main.rs', 'w') as f:
    f.writelines(lines)
