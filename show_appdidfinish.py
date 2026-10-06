with open('src/main.rs', 'r') as f:
    content = f.read()
idx = content.find("fn application_did_finish_launching")
print(content[idx:idx+3500])
