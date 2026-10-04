use objc2::{define_class, rc::Retained, ClassType, msg_send, DeclaredClass};
use objc2_app_kit::{NSOutlineViewDataSource, NSOutlineViewDelegate, NSOutlineView};
use objc2_foundation::{NSObject, NSObjectProtocol, NSInteger};
use objc2::runtime::AnyObject;

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "WorkspaceDelegate"]
    struct WorkspaceDelegate;
    unsafe impl NSObjectProtocol for WorkspaceDelegate {}
    unsafe impl NSOutlineViewDataSource for WorkspaceDelegate {
        #[unsafe(method(outlineView:numberOfChildrenOfItem:))]
        fn outlineView_numberOfChildrenOfItem(&self, _outline_view: &NSOutlineView, item: Option<&AnyObject>) -> NSInteger {
            0
        }
    }
);
