import re
with open('src/workspace_delegate.rs', 'r') as f:
    content = f.read()

start_idx = content.find("#[unsafe(method(downloadItem:))]")
end_idx = content.find("impl WorkspaceDelegate {")

if start_idx != -1 and end_idx != -1:
    content = content[:start_idx] + "    }\n);\n\n" + content[end_idx:]

with open('src/workspace_delegate.rs', 'w') as f:
    f.write(content)
