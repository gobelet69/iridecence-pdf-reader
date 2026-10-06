import re

with open('src/main.rs', 'r') as f:
    content = f.read()

content = content.replace("if let Some(empty) = existing_empty {", "if let Some(empty) = existing_empty.as_ref() {")

with open('src/main.rs', 'w') as f:
    f.write(content)
