import re

with open('src/main.rs', 'r') as f:
    content = f.read()

# Fix imageViewWithImage
content = content.replace("NSImageView::imageViewWithImage(&img)", "NSImageView::imageViewWithImage(&img, mtm)")

# Fix NSImageSymbolScaleLarge import
content = content.replace("NSImageSymbolScaleLarge", "NSImageSymbolScale")
content = content.replace("NSFontWeightRegular", "")

# Remove setSymbolConfiguration for now to avoid enum issues, just let it be default scale
content = re.sub(r'let _: \(\) = objc2::msg_send!\[&img_view, setSymbolConfiguration: .*?\];', '', content)

with open('src/main.rs', 'w') as f:
    f.write(content)
