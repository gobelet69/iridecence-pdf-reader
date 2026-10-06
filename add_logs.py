import re

with open("src/workspace.rs", "r") as f:
    ws = f.read()

# Add log to load_children
ws = ws.replace("self.ivars().is_loading.set(true);", "self.ivars().is_loading.set(true); println!(\"load_children started for {:?}\", self.ivars().path.borrow());")

# Add log to thread
thread_start = "std::thread::spawn(move || {"
ws = ws.replace(thread_start, thread_start + "\n            println!(\"Thread started\");")

thread_end = "let obj_raw = node_ptr as *mut objc2::runtime::AnyObject;"
ws = ws.replace(thread_end, "println!(\"Thread finished, posting finishLoading to {:?}\", node_ptr);\n            " + thread_end)

# Add log to finishLoading
finish_start = "fn finishLoading(&self) {"
ws = ws.replace(finish_start, finish_start + "\n        println!(\"finishLoading started for {:?}\", self.ivars().path.borrow());")

finish_end = "nc.postNotificationName_object(name, Some(self));"
ws = ws.replace(finish_end, finish_end + "\n            println!(\"Posted FileNodeDidLoadChildren for {:?}\", self.ivars().path.borrow());")

with open("src/workspace.rs", "w") as f:
    f.write(ws)

with open("src/workspace_delegate.rs", "r") as f:
    wd = f.read()

# Add log to nodeDidLoadChildren
node_did_load = "fn nodeDidLoadChildren(&self, notification: &objc2_foundation::NSNotification) {"
wd = wd.replace(node_did_load, node_did_load + "\n        println!(\"nodeDidLoadChildren received notification\");")

is_root_log = "if is_root {"
wd = wd.replace(is_root_log, "println!(\"is_root: {}\", is_root);\n                " + is_root_log)

with open("src/workspace_delegate.rs", "w") as f:
    f.write(wd)
