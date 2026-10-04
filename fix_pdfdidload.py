with open('src/main.rs', 'r') as f:
    content = f.read()

content = content.replace("objc2::sel!(pdfReady:)", "objc2::sel!(pdfDidLoad:)")

with open('src/main.rs', 'w') as f:
    f.write(content)

