import re

with open('src/main.rs', 'r') as f:
    content = f.read()

# We need to remove the loop from `openPdf`
# The lines are from `let is_icloud` to `});` just before `std::thread::spawn(move || {`?
# No, let's just strip the polling loop from the `thread::spawn` in `openPdf`.

# I will just regex replace the polling loop with nothing.
start_idx = content.find('// Polling loop for iCloud')
if start_idx != -1:
    end_idx = content.find('// 3. Init PDFDocument', start_idx)
    if end_idx != -1:
        content = content[:start_idx] + content[end_idx:]

with open('src/main.rs', 'w') as f:
    f.write(content)
print("Stripped old loop")
