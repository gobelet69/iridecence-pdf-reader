with open('src/main.rs', 'r') as f:
    content = f.read()

content = content.replace("startDownloadingUbiquitousItemAtURL: &nsurl, error: &mut err", "startDownloadingUbiquitousItemAtURL: &*nsurl, error: &mut err")

with open('src/main.rs', 'w') as f:
    f.write(content)
