import sys

with open('src/palette.rs', 'r') as f:
    pal = f.read()

if "    }\n);" in pal:
    print("Already has closing brace?")
else:
    pal = pal.replace("\n);", "\n    }\n);")

with open('src/palette.rs', 'w') as f:
    f.write(pal)

