with open('src/main.rs', 'r') as f:
    content = f.read()

import re

# find workspaceDoubleClicked
match1 = re.search(r"#\[unsafe\(method\(workspaceDoubleClicked:\)\)\]\s+fn workspace_double_clicked\(&self, sender: Option<&AnyObject>\) \{(.*?)\n        \}\n\n        \n        #\[unsafe\(method\(pdfDidLoad:\)\)\]", content, re.DOTALL)
if match1:
    body = match1.group(1)
    
    # replace workspaceSelectionChanged body
    content = re.sub(r"#\[unsafe\(method\(workspaceSelectionChanged:\)\)\]\s+fn workspace_selection_changed\(&self, sender: Option<&AnyObject>\) \{.*?(?=\n        fn application_open_urls|\n        #\[unsafe\(method\(application:openURLs:\)\)\])", 
           f"#[unsafe(method(workspaceSelectionChanged:))]\n        fn workspace_selection_changed(&self, sender: Option<&AnyObject>) {{{body}\n        }}\n\n", content, flags=re.DOTALL)

with open('src/main.rs', 'w') as f:
    f.write(content)

