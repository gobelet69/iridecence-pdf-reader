with open('src/main.rs', 'r') as f:
    content = f.read()

opens = content.count('{')
closes = content.count('}')

print(f"Opens: {opens}, Closes: {closes}")
