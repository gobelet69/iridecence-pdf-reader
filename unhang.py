import re
with open('src/workspace_delegate.rs', 'r') as f:
    content = f.read()

bad_match = """                let diag_state = match icloud_status {
                    0 => "Local",
                    1 => "CloudOnly",
                    2 => "Downloading",
                    4 => "Failed",
                    _ => "Unknown",
                };
                let diag_icon = match icloud_status {
                    0 => "NONE",
                    1 => "Cloud",
                    2 => "Spinner",
                    4 => "Failed",
                    _ => "NONE",
                };"""

good_code = """                let diag_state = if icloud_status == 0 { "Local" } else if icloud_status == 1 { "CloudOnly" } else if icloud_status == 2 { "Downloading" } else if icloud_status == 4 { "Failed" } else { "Unknown" };
                let diag_icon = if icloud_status == 0 { "NONE" } else if icloud_status == 1 { "Cloud" } else if icloud_status == 2 { "Spinner" } else if icloud_status == 4 { "Failed" } else { "NONE" };"""

content = content.replace(bad_match, good_code)

with open('src/workspace_delegate.rs', 'w') as f:
    f.write(content)
