use objc2::{define_class, rc::Retained};
use objc2_app_kit::{NSOutlineViewDataSource, NSOutlineViewDelegate, NSOutlineView, NSTableColumn, NSView};
use objc2_foundation::{NSObject, NSObjectProtocol, NSInteger};
use objc2::runtime::AnyObject;

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "WorkspaceDelegate"]
    struct WorkspaceDelegate;
    unsafe impl NSObjectProtocol for WorkspaceDelegate {}
    unsafe impl NSOutlineViewDataSource for WorkspaceDelegate {
        #[unsafe(method(outlineView:child:ofItem:))]
        fn outlineView_child_ofItem(&self, _outline_view: &NSOutlineView, index: NSInteger, item: Option<&AnyObject>) -> Option<Retained<AnyObject>> {
            None
        }
    }
);
