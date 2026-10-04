use objc2::{define_class, rc::Retained};
use objc2_foundation::{NSObject, NSObjectProtocol};
use objc2_app_kit::{NSOutlineViewDataSource, NSOutlineView};

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "TestClass"]
    pub struct TestClass;
);
