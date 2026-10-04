with open("src/main.rs", "r") as f:
    code = f.read()

code = code.replace(
    "let outline_raw = Retained::into_raw(objc2::rc::Retained::retain(outline)) as usize;",
    "let outline_raw = outline as *const _ as *mut NSOutlineView as usize;"
)

with open("src/main.rs", "w") as f:
    f.write(code)
