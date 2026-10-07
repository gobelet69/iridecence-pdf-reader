import sys

with open('src/palette.rs', 'r') as f:
    content = f.read()

content = content.replace(
    "use objc2::{define_class, msg_send, msg_send_id, ClassType, sel};",
    "use objc2::{define_class, msg_send, msg_send_id, ClassType, sel, MainThreadOnly};"
)

with open('src/palette.rs', 'w') as f:
    f.write(content)
