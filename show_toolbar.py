with open('src/main.rs', 'r') as f:
    content = f.read()
start = content.find("fn toolbar_itemForItemIdentifier")
end = content.find("fn toolbarDefaultItemIdentifiers")
print(content[start:end])
