with open('src/main.rs', 'r') as f:
    content = f.read()

idx = content.find("fn open_pdf")
print(content[idx:idx+800])
