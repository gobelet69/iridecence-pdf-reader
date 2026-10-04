with open("src/workspace_delegate.rs", "r") as f:
    code = f.read()

code = code.replace("        this\n    }\n    }", "        this\n    }")

with open("src/workspace_delegate.rs", "w") as f:
    f.write(code)
