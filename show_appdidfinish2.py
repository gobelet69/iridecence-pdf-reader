with open('src/main.rs', 'r') as f:
    content = f.read()
idx = content.find("thumb_view.setThumbnailSize(NSSize::new(100.0, 140.0));")
print(content[idx:idx+2500])
