with open('src/main.rs', 'r') as f:
    content = f.read()

idx = content.find("std::thread::spawn(move || {", content.find("fn workspace_selection_changed"))
print(content[idx:idx+1500])
