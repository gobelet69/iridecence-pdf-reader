with open('src/main.rs', 'r') as f:
    content = f.read()
idx = content.find("outline_scroll.setHasVerticalSc")
print(content[idx:idx+2500])
