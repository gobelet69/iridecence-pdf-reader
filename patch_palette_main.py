import sys

def modify_main():
    with open('src/main.rs', 'r') as f:
        content = f.read()

    # 1. Add Ivars
    ivars_target = "history_index: std::cell::Cell<usize>,\n    ignore_history_update: std::cell::Cell<bool>,\n}"
    ivars_replacement = """history_index: std::cell::Cell<usize>,
    ignore_history_update: std::cell::Cell<bool>,
    
    palette_panel: std::cell::OnceCell<objc2::rc::Retained<objc2_app_kit::NSPanel>>,
    palette_delegate: std::cell::OnceCell<objc2::rc::Retained<crate::palette::CommandPaletteDelegate>>,
}"""
    if ivars_target in content:
        content = content.replace(ivars_target, ivars_replacement)
    else:
        print("Ivars not found!")
    
    # 2. Modify `application_did_finish_launching` to create the palette
    init_target = "self.ivars().pdf_view.set(pdf_view).unwrap();"
    init_replacement = """self.ivars().pdf_view.set(pdf_view).unwrap();
            
            let (palette_panel, palette_delegate) = crate::palette::create_palette(mtm);
            let _ = self.ivars().palette_panel.set(palette_panel);
            let _ = self.ivars().palette_delegate.set(palette_delegate);
"""
    if init_target in content:
        content = content.replace(init_target, init_replacement)
    
    # 3. Rewrite `quick_open`, `command_palette`, `go_to_page`
    def replace_method(name, new_impl):
        nonlocal content
        start_str = f"#[unsafe(method({name}:))]"
        end_str = f"fn {name}(&self, _sender: Option<&AnyObject>)"
        if start_str not in content:
            return
        
        # Find start
        s_idx = content.find(start_str)
        # Find brace end
        fn_start = content.find("{", s_idx)
        brace_count = 1
        e_idx = fn_start + 1
        while brace_count > 0 and e_idx < len(content):
            if content[e_idx] == '{': brace_count += 1
            elif content[e_idx] == '}': brace_count -= 1
            e_idx += 1
            
        content = content[:s_idx] + new_impl + content[e_idx:]

    quick_open_new = """#[unsafe(method(quickOpen:))]
        fn quick_open(&self, _sender: Option<&AnyObject>) {
            if let Some(panel) = self.ivars().palette_panel.get() {
                if let Some(del) = self.ivars().palette_delegate.get() {
                    if let Some(tf) = del.ivars().text_field.get() {
                        unsafe { tf.setStringValue(&objc2_foundation::NSString::from_str("")) };
                        del.update_search("");
                        unsafe { panel.makeKeyAndOrderFront(None) };
                        if let Some(window) = self.ivars().window.get() {
                            unsafe { panel.center() };
                        }
                    }
                }
            }
        }"""
    
    command_palette_new = """#[unsafe(method(commandPalette:))]
        fn command_palette(&self, _sender: Option<&AnyObject>) {
            if let Some(panel) = self.ivars().palette_panel.get() {
                if let Some(del) = self.ivars().palette_delegate.get() {
                    if let Some(tf) = del.ivars().text_field.get() {
                        unsafe { tf.setStringValue(&objc2_foundation::NSString::from_str(">")) };
                        del.update_search(">");
                        unsafe { panel.makeKeyAndOrderFront(None) };
                        if let Some(window) = self.ivars().window.get() {
                            unsafe { panel.center() };
                        }
                    }
                }
            }
        }"""
        
    go_to_page_new = """#[unsafe(method(goToPage:))]
        fn go_to_page(&self, _sender: Option<&AnyObject>) {
            if let Some(panel) = self.ivars().palette_panel.get() {
                if let Some(del) = self.ivars().palette_delegate.get() {
                    if let Some(tf) = del.ivars().text_field.get() {
                        unsafe { tf.setStringValue(&objc2_foundation::NSString::from_str("p:")) };
                        del.update_search("p:");
                        unsafe { panel.makeKeyAndOrderFront(None) };
                        if let Some(window) = self.ivars().window.get() {
                            unsafe { panel.center() };
                        }
                    }
                }
            }
        }"""
        
    replace_method("quickOpen", quick_open_new)
    replace_method("commandPalette", command_palette_new)
    replace_method("goToPage", go_to_page_new)
    
    with open('src/main.rs', 'w') as f:
        f.write(content)
        
    print("Patched main.rs for palette")

modify_main()
