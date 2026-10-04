import re

with open("src/workspace.rs", "r") as f:
    ws = f.read()

ws = ws.replace("self.ivars().is_loading.set(true); println!(\"load_children started for {:?}\", self.ivars().path.borrow());", "self.ivars().is_loading.set(true);")
ws = ws.replace("std::thread::spawn(move || {\n            println!(\"Thread started\");", "std::thread::spawn(move || {")
ws = ws.replace("println!(\"Thread finished, posting finishLoading to {:?}\", node_ptr);\n            let obj_raw", "let obj_raw")
ws = ws.replace("fn finishLoading(&self) {\n        println!(\"finishLoading started for {:?}\", self.ivars().path.borrow());", "fn finishLoading(&self) {")
ws = ws.replace("nc.postNotificationName_object(name, Some(self));\n            println!(\"Posted FileNodeDidLoadChildren for {:?}\", self.ivars().path.borrow());", "nc.postNotificationName_object(name, Some(self));")

with open("src/workspace.rs", "w") as f:
    f.write(ws)

with open("src/workspace_delegate.rs", "r") as f:
    wd = f.read()

wd = wd.replace("fn nodeDidLoadChildren(&self, notification: &objc2_foundation::NSNotification) {\n        println!(\"nodeDidLoadChildren received!\");", "fn nodeDidLoadChildren(&self, notification: &objc2_foundation::NSNotification) {")
wd = wd.replace("if let Some(outline_view) = weak_view.load() {\n                println!(\"outline_view loaded!\");", "if let Some(outline_view) = weak_view.load() {")
wd = wd.replace("println!(\"is_root: {}\", is_root);\n                if is_root {", "if is_root {")

with open("src/workspace_delegate.rs", "w") as f:
    f.write(wd)

