with open('src/main.rs', 'r') as f:
    content = f.read()

idx = content.find("fn workspace_selection_changed")
print(content[idx:idx+2500])
