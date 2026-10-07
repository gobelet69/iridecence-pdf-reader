import sys

def modify_main():
    with open('src/main.rs', 'r') as f:
        content = f.read()
    
    # Let's replace the Right Sidebar setup in main.rs
    
    # 1. We need to find where RIGHT SIDEBAR starts
    start_marker = "// RIGHT SIDEBAR (Outline + Pages)"
    
    # 2. End marker is roughly before `let navigation_vc = NSViewController::new(mtm);`
    end_marker = "let navigation_vc = NSViewController::new(mtm);"
    
    if start_marker not in content or end_marker not in content:
        print("Markers not found")
        return
    
    start_idx = content.find(start_marker)
    end_idx = content.find(end_marker)
    
    # The new code to insert
    new_code = """// RIGHT SIDEBAR (Outline + Pages + Search)
            let pdf_outline_del = crate::pdf_outline::PdfOutlineDelegate::new(mtm, pdf_view.clone());
            let right_outline_view = unsafe { NSOutlineView::initWithFrame(NSOutlineView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            let right_col = unsafe { NSTableColumn::initWithIdentifier(NSTableColumn::alloc(mtm), objc2_foundation::ns_string!("OutlineColumn")) };
            unsafe {
                right_col.setWidth(230.0);
                right_outline_view.addTableColumn(&right_col);
                right_outline_view.setOutlineTableColumn(Some(&right_col));
                right_outline_view.setHeaderView(None);

                let del_ptr: *const crate::pdf_outline::PdfOutlineDelegate = &*pdf_outline_del;
                let _: () = msg_send![&right_outline_view, setDelegate: del_ptr];
                let _: () = msg_send![&right_outline_view, setDataSource: del_ptr];
            }

            let right_outline_scroll = unsafe { NSScrollView::initWithFrame(NSScrollView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            unsafe {
                right_outline_scroll.setHasVerticalScroller(true);
                right_outline_scroll.setDocumentView(Some(&right_outline_view));
                right_outline_scroll.setDrawsBackground(false);
                right_outline_scroll.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
                right_outline_scroll.setHidden(true); // default to Pages
            }
            
            // Search UI
            let pdf_search_del = crate::pdf_search::PdfSearchDelegate::new(mtm, pdf_view.clone());
            let search_outline_view = unsafe { NSOutlineView::initWithFrame(NSOutlineView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            let search_col = unsafe { NSTableColumn::initWithIdentifier(NSTableColumn::alloc(mtm), objc2_foundation::ns_string!("SearchColumn")) };
            unsafe {
                search_col.setWidth(230.0);
                search_outline_view.addTableColumn(&search_col);
                search_outline_view.setOutlineTableColumn(Some(&search_col));
                search_outline_view.setHeaderView(None);
                
                let s_del_ptr: *const crate::pdf_search::PdfSearchDelegate = &*pdf_search_del;
                let _: () = msg_send![&search_outline_view, setDelegate: s_del_ptr];
                let _: () = msg_send![&search_outline_view, setDataSource: s_del_ptr];
            }
            pdf_search_del.ivars().outline_view.set(search_outline_view.clone()).unwrap();
            
            let search_scroll = unsafe { NSScrollView::initWithFrame(NSScrollView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            unsafe {
                search_scroll.setHasVerticalScroller(true);
                search_scroll.setDocumentView(Some(&search_outline_view));
                search_scroll.setDrawsBackground(false);
                search_scroll.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
                search_scroll.setHidden(true);
            }
            
            let search_field = unsafe { NSTextField::textFieldWithString(objc2_foundation::ns_string!(""), mtm) };
            unsafe {
                let s_del_ptr: *const crate::pdf_search::PdfSearchDelegate = &*pdf_search_del;
                let _: () = msg_send![&search_field, setDelegate: s_del_ptr];
                search_field.setPlaceholderString(Some(objc2_foundation::ns_string!("Search...")));
                search_field.setTranslatesAutoresizingMaskIntoConstraints(false);
            }
            
            let search_container = unsafe { NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 700.0))) };
            unsafe {
                search_container.setAutoresizingMask(objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable);
                search_container.addSubview(&search_field);
                search_container.addSubview(&search_scroll);
                search_scroll.setTranslatesAutoresizingMaskIntoConstraints(false);
                
                search_field.topAnchor().constraintEqualToAnchor_constant(&search_container.topAnchor(), 8.0).setActive(true);
                search_field.leadingAnchor().constraintEqualToAnchor_constant(&search_container.leadingAnchor(), 8.0).setActive(true);
                search_field.trailingAnchor().constraintEqualToAnchor_constant(&search_container.trailingAnchor(), -8.0).setActive(true);
                
                search_scroll.topAnchor().constraintEqualToAnchor_constant(&search_field.bottomAnchor(), 8.0).setActive(true);
                search_scroll.leadingAnchor().constraintEqualToAnchor(&search_container.leadingAnchor()).setActive(true);
                search_scroll.trailingAnchor().constraintEqualToAnchor(&search_container.trailingAnchor()).setActive(true);
                search_scroll.bottomAnchor().constraintEqualToAnchor(&search_container.bottomAnchor()).setActive(true);
                
                search_container.setHidden(true);
            }

            // Using NSStackView for right_container
            let right_container = unsafe { NSStackView::stackViewWithViews(&NSArray::from_slice(&[]), mtm) };
            unsafe {
                right_container.setOrientation(objc2_app_kit::NSUserInterfaceLayoutOrientation::Vertical);
                right_container.setSpacing(0.0);
                right_container.setDistribution(objc2_app_kit::NSStackViewDistribution::Fill);
                right_container.setAlignment(objc2_app_kit::NSLayoutAttribute::CenterX);
            }

            let segmented = unsafe { NSSegmentedControl::segmentedControlWithLabels_trackingMode_target_action(
                &NSArray::from_slice(&[objc2_foundation::ns_string!("Pages"), objc2_foundation::ns_string!("Outline"), objc2_foundation::ns_string!("Search")]),
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
                    search_container.setHidden(true);
                } else if selected_idx == 2 {
                    right_outline_scroll.setHidden(true);
                    thumb_view.setHidden(true);
                    search_container.setHidden(false);
                } else {
                    right_outline_scroll.setHidden(true);
                    thumb_view.setHidden(false);
                    search_container.setHidden(true);
                }

                let seg_wrapper = NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 41.0)));
                seg_wrapper.addSubview(&segmented);
                seg_wrapper.setTranslatesAutoresizingMaskIntoConstraints(false);
                
                segmented.setTranslatesAutoresizingMaskIntoConstraints(false);
                segmented.topAnchor().constraintEqualToAnchor_constant(&seg_wrapper.topAnchor(), 8.0).setActive(true);
                segmented.bottomAnchor().constraintEqualToAnchor_constant(&seg_wrapper.bottomAnchor(), -8.0).setActive(true);
                segmented.leadingAnchor().constraintEqualToAnchor_constant(&seg_wrapper.leadingAnchor(), 12.0).setActive(true);
                segmented.trailingAnchor().constraintEqualToAnchor_constant(&seg_wrapper.trailingAnchor(), -12.0).setActive(true);

                let separator = NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 1.0)));
                separator.setWantsLayer(true);
                if let Some(layer) = separator.layer() {
                    let cg = objc2_app_kit::NSColor::separatorColor().CGColor(); let ptr = Retained::as_ptr(&cg); let _: () = msg_send![&layer, setBackgroundColor: ptr];
                }
                separator.setTranslatesAutoresizingMaskIntoConstraints(false);
                separator.heightAnchor().constraintEqualToConstant(1.0).setActive(true);

                let content_wrapper = NSView::initWithFrame(NSView::alloc(mtm), NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(230.0, 660.0)));
                content_wrapper.addSubview(&thumb_view);
                content_wrapper.addSubview(&right_outline_scroll);
                content_wrapper.addSubview(&search_container);
                
                thumb_view.setTranslatesAutoresizingMaskIntoConstraints(false);
                right_outline_scroll.setTranslatesAutoresizingMaskIntoConstraints(false);
                search_container.setTranslatesAutoresizingMaskIntoConstraints(false);
                
                for v in [&thumb_view, &right_outline_scroll, &search_container] {
                    v.topAnchor().constraintEqualToAnchor(&content_wrapper.topAnchor()).setActive(true);
                    v.bottomAnchor().constraintEqualToAnchor(&content_wrapper.bottomAnchor()).setActive(true);
                    v.leadingAnchor().constraintEqualToAnchor(&content_wrapper.leadingAnchor()).setActive(true);
                    v.trailingAnchor().constraintEqualToAnchor(&content_wrapper.trailingAnchor()).setActive(true);
                }

                right_container.addView_inGravity(&seg_wrapper, objc2_app_kit::NSStackViewGravity::Top);
                right_container.addView_inGravity(&separator, objc2_app_kit::NSStackViewGravity::Top);
                right_container.addView_inGravity(&content_wrapper, objc2_app_kit::NSStackViewGravity::Top);
                
                seg_wrapper.widthAnchor().constraintEqualToAnchor(&right_container.widthAnchor()).setActive(true);
                separator.widthAnchor().constraintEqualToAnchor(&right_container.widthAnchor()).setActive(true);
                content_wrapper.widthAnchor().constraintEqualToAnchor(&right_container.widthAnchor()).setActive(true);
            }

            unsafe {
                let nc = objc2_foundation::NSNotificationCenter::defaultCenter();
                nc.addObserver_selector_name_object(
                    &*(self as *const AppDelegate as *const AnyObject),
                    objc2::sel!(thumbnailFrameChanged:),
                    Some(objc2_app_kit::NSViewFrameDidChangeNotification),
                    Some(&*right_container as &AnyObject) // Observe container instead
                );
                right_container.setPostsFrameChangedNotifications(true);
            }
            
            let _ = self.ivars().right_sidebar_search_container.set(search_container);
            let _ = self.ivars().search_field.set(search_field);
            let _ = self.ivars().pdf_search_delegate.set(pdf_search_del);
            let _ = self.ivars().right_sidebar_segmented.set(segmented.clone());
            
            """
    
    new_content = content[:start_idx] + new_code + "\n            " + content[end_idx:]
    
    # Also we need to fix `switchSidebar:` in main.rs
    
    switch_sidebar_start = "fn switch_sidebar(&self, sender: Option<&AnyObject>) {"
    switch_sidebar_end = "updateOutlineEmptyState] };"
    if switch_sidebar_start in new_content:
        s_idx = new_content.find(switch_sidebar_start)
        e_idx = new_content.find(switch_sidebar_end, s_idx)
        
        # we will replace it with a new one
        new_switch = """fn switch_sidebar(&self, sender: Option<&AnyObject>) {
            if let Some(sender) = sender {
                if let Some(segmented) = sender.downcast_ref::<NSSegmentedControl>() {
                    let index = unsafe { segmented.selectedSegment() };
                    unsafe {
                        let defaults = objc2_foundation::NSUserDefaults::standardUserDefaults();
                        defaults.setInteger_forKey(index, objc2_foundation::ns_string!("RightSidebarMode"));
                    }
                    if let Some(outline_scroll) = self.ivars().right_sidebar_outline_scroll.get() {
                        let thumb_view = self.ivars().thumb_view.get().unwrap();
                        let search_container = self.ivars().right_sidebar_search_container.get();

                        if index == 0 {
                            unsafe { outline_scroll.setHidden(true) };
                            if let Some(search_container) = search_container {
                                unsafe { search_container.setHidden(true) };
                            }
                            unsafe { thumb_view.setHidden(false) };
                        } else if index == 1 {
                            unsafe { outline_scroll.setHidden(false) };
                            if let Some(search_container) = search_container {
                                unsafe { search_container.setHidden(true) };
                            }
                            unsafe { thumb_view.setHidden(true) };
                        } else {
                            unsafe { outline_scroll.setHidden(true) };
                            if let Some(search_container) = search_container {
                                unsafe { search_container.setHidden(false) };
                            }
                            unsafe { thumb_view.setHidden(true) };
                        }

                        let _: () = unsafe { objc2::msg_send![self, updateOutlineEmptyState] };"""
        
        new_content = new_content[:s_idx] + new_switch + new_content[e_idx + len(switch_sidebar_end):]

    with open('src/main.rs', 'w') as f:
        f.write(new_content)
        
    print("Modifications applied successfully")

if __name__ == "__main__":
    modify_main()
