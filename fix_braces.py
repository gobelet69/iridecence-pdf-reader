with open('src/main.rs', 'r') as f:
    content = f.read()

idx = content.find("fn workspace_selection_changed")
idx2 = content.find("fn pdfDidLoad:", idx)
print(content[idx:idx2])
