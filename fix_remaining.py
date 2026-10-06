import re

with open('src/main.rs', 'r') as f:
    content = f.read()

# Fix 1: Ensure switch_sidebar calls updateOutlineEmptyState
old_switch = """                            unsafe { thumb_view.setHidden(true) };
                        }
                    }
                }
            }
        }"""
new_switch = """                            unsafe { thumb_view.setHidden(true) };
                        }
                        let _: () = unsafe { objc2::msg_send![self, updateOutlineEmptyState] };
                    }
                }
            }
        }"""
content = content.replace(old_switch, new_switch)

# Fix Right Sidebar Width (200.0 -> 230.0 for all right panel elements)
content = content.replace("NSSize::new(200.0, 700.0)", "NSSize::new(230.0, 700.0)")
content = content.replace("NSSize::new(200.0, 660.0)", "NSSize::new(230.0, 660.0)")
content = content.replace("NSSize::new(200.0, 40.0)", "NSSize::new(230.0, 41.0)")
content = content.replace("NSSize::new(200.0, 1.0)", "NSSize::new(230.0, 1.0)")
content = content.replace("NSSize::new(200.0, 24.0)", "NSSize::new(230.0, 24.0)")
content = content.replace("right_col.setWidth(200.0);", "right_col.setWidth(230.0);")

# Fix Header Padding and Separator
old_seg_wrapper = """                let seg_wrapper = NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 41.0)));
                seg_wrapper.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewMinYMargin);
                seg_wrapper.addSubview(&segmented);
                // Center it (manual approximation)
                segmented.setFrameOrigin(NSPoint::new(40.0, 8.0));
                segmented.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewMinXMargin | objc2_app_kit::NSAutoresizingMaskOptions::ViewMaxXMargin);"""
                
new_seg_wrapper = """                let seg_wrapper = NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 41.0)));
                seg_wrapper.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewMinYMargin);
                seg_wrapper.addSubview(&segmented);
                
                // Add 8-10pt top padding, 10-12pt horizontal padding, 8pt bottom padding.
                // 41pt height - 8pt top - 8pt bottom = 25pt.
                segmented.setFrame(NSRect::new(NSPoint::new(12.0, 8.0), NSSize::new(230.0 - 24.0, 25.0)));
                segmented.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable);"""
                
content = content.replace(old_seg_wrapper, new_seg_wrapper)


# Fix persistance of sidebar mode (use UserDefaults)
old_seg_init = """            let segmented = unsafe { NSSegmentedControl::segmentedControlWithLabels_trackingMode_target_action(
                &NSArray::from_slice(&[objc2_foundation::ns_string!("Pages"), objc2_foundation::ns_string!("Outline")]),
                NSSegmentSwitchTracking::SelectOne,
                Some(unsafe { &*(self as *const AppDelegate as *const AnyObject) }),
                Some(objc2::sel!(switchSidebar:)),
                mtm
            ) };
            unsafe {
                segmented.setSelected_forSegment(true, 0);"""

new_seg_init = """            let segmented = unsafe { NSSegmentedControl::segmentedControlWithLabels_trackingMode_target_action(
                &NSArray::from_slice(&[objc2_foundation::ns_string!("Pages"), objc2_foundation::ns_string!("Outline")]),
                NSSegmentSwitchTracking::SelectOne,
                Some(unsafe { &*(self as *const AppDelegate as *const AnyObject) }),
                Some(objc2::sel!(switchSidebar:)),
                mtm
            ) };
            unsafe {
                let defaults = objc2_foundation::NSUserDefaults::standardUserDefaults();
                let selected_idx = defaults.integerForKey(objc2_foundation::ns_string!("RightSidebarMode"));
                segmented.setSelected_forSegment(true, selected_idx);
                
                if selected_idx == 1 {
                    right_outline_scroll.setHidden(false);
                    thumb_view.setHidden(true);
                } else {
                    right_outline_scroll.setHidden(true);
                    thumb_view.setHidden(false);
                }"""

content = content.replace(old_seg_init, new_seg_init)

# In switchSidebar, save to defaults
old_switch2 = """                    if let Some(outline_scroll) = self.ivars().right_sidebar_outline_scroll.get() {"""
new_switch2 = """                    unsafe {
                        let defaults = objc2_foundation::NSUserDefaults::standardUserDefaults();
                        defaults.setInteger_forKey(index, objc2_foundation::ns_string!("RightSidebarMode"));
                    }
                    if let Some(outline_scroll) = self.ivars().right_sidebar_outline_scroll.get() {"""
content = content.replace(old_switch2, new_switch2)

with open('src/main.rs', 'w') as f:
    f.write(content)
