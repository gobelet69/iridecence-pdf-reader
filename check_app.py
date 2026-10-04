with open('src/main.rs', 'r') as f:
    content = f.read()

idx1 = content.find("fn application_did_finish_launching")
idx2 = content.find(");", idx1)

body = content[idx1:idx2]

opens = body.count('{')
closes = body.count('}')

print(f"Opens: {opens}, Closes: {closes}")
