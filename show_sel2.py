with open('src/main.rs', 'r') as f:
    content = f.read()

idx = content.find("println!(\"[PDF TIMING] PDFDocument::initWithURL: {:?}\", t2 - t1);")
end_idx = content.find("        }", idx)
print(content[idx:idx+1500])
