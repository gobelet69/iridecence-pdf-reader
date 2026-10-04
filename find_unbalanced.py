with open('src/main.rs', 'r') as f:
    content = f.read()

idx1 = content.find("impl AppDelegate {")
idx2 = content.find("\n);", idx1)

body = content[idx1:idx2]
opens = body.count('{')
closes = body.count('}')

print(f"Total opens: {opens}, Total closes: {closes}")

import re
funcs = re.findall(r"(#\[unsafe\(method\(.*?\)\]\s+fn.*?)\n\n", body, re.DOTALL)
if not funcs:
    funcs = body.split("#[unsafe(method(")

for i, fn in enumerate(funcs):
    o = fn.count('{')
    c = fn.count('}')
    if o != c:
        print(f"Function {i} unbalanced! {o} opens, {c} closes. Starts with: {fn[:50].strip()}")
