use objc2::{define_class, rc::Retained};
use objc2_app_kit::{NSOutlineViewDataSource, NSOutlineViewDelegate, NSOutlineView, NSTableColumn, NSView};
use objc2_foundation::{NSObject, NSObjectProtocol, NSInteger};
use objc2::runtime::AnyObject;

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "WorkspaceDelegate"]
    struct WorkspaceDelegate;
    unsafe impl NSObjectProtocol for WorkspaceDelegate {}
    unsafe impl NSOutlineViewDelegate for WorkspaceDelegate {
        #[unsafe(method(outlineView:viewForTableColumn:item:))]
        fn outlineView_viewForTableColumn_item(&self, outline_view: &NSOutlineView, table_column: Option<&NSTableColumn>, item: &AnyObject) -> Option<Retained<NSView>> {
            None
        }
    }
);
