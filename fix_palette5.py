import sys

with open('src/palette.rs', 'r') as f:
    pal = f.read()

target = """        fn controlTextDidChange(&self, _notification: &NSNotification) {
            if let Some(tf) = self.ivars().text_field.get() {
                let s = unsafe { tf.stringValue().to_string() };
                self.update_search(&s);
            }
        }
        
        
            
            false
        }"""
replacement = """        fn controlTextDidChange(&self, _notification: &NSNotification) {
            if let Some(tf) = self.ivars().text_field.get() {
                let s = unsafe { tf.stringValue().to_string() };
                self.update_search(&s);
            }
        }"""

pal = pal.replace(target, replacement)

with open('src/palette.rs', 'w') as f:
    f.write(pal)
