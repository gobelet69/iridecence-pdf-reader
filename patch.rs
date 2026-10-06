    #[unsafe(method(outlineViewSelectionDidChange:))]
    fn outlineViewSelectionDidChange(&self, notification: &objc2_foundation::NSNotification) {
        let outline_view: objc2::rc::Retained<NSOutlineView> = unsafe { objc2::msg_send![notification, object] };
        
        let target: Option<objc2::rc::Retained<objc2::runtime::AnyObject>> = unsafe { objc2::msg_send![&outline_view, target] };
        if let Some(target) = target {
            let _: () = unsafe { objc2::msg_send![&target, workspaceSelectionChanged: &*outline_view] };
        }
    }
