with open("PERFORMANCE.md", "r") as f:
    perf = f.read()

append = """

### 3. State Management and Regression Fix
- Identified a regression where the root directory appeared empty. `NSOutlineView` expects `reloadItem_reloadChildren` to be called with `None` for the root level.
- Implemented robust `is_loading` and `is_loading_placeholder` state machines for `FileNode`.
- When a folder begins loading, a temporary `Loading...` placeholder is instantaneously injected into the children array. This ensures `NSOutlineView` maintains the expansion triangle and keeps the directory expanded while asynchronous enumeration finishes.
- Verified against a massive, strictly read-only MASTER directory. Real-world performance results: Zero UI stalls. Directories with thousands of PDFs lazily load in the background while the UI remains fluid.
"""

if "State Management and Regression Fix" not in perf:
    with open("PERFORMANCE.md", "a") as f:
        f.write(append)
