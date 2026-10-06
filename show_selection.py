with open('src/main.rs', 'r') as f:
    content = f.read()

import re
match = re.search(r"fn workspaceSelectionChanged.*?(?=\n\n|\Z)", content, re.DOTALL)
if match:
    print(match.group(0))
else:
    # Just find where it is
    idx = content.find("workspaceSelectionChanged")
    if idx != -1:
        print(content[idx-200:idx+800])
    else:
        print("Not found.")
