//! https://github.com/neovide/neovide/blob/38fd9d75419dc00dd2733965f7aa753d759021af/src/window/macos.rs#L512
//! and I also took inspiration from zed
//! NOTE still placeholders
//! TODO proper integration

use objc2::{ClassType, DeclaredClass, declare_class, msg_send_id, mutability, rc::Retained, sel};
use objc2_app_kit::{NSApplication, NSEvent, NSEventModifierFlags, NSMenu, NSMenuItem};
use objc2_foundation::{MainThreadMarker, NSObject, NSProcessInfo, ns_string};
use std::str;

pub(crate) fn show_nat_menu(_ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // nothing to do here
}

pub(crate) fn init_nat_menu() {
    let mtm = MainThreadMarker::new().expect("Menu must be created on the main thread");
    Box::leak(Box::new(Menu::new(mtm)));
}

#[derive(Debug)]
struct Menu {
    quit_handler: Retained<QuitHandler>,
}

impl Menu {
    fn new(mtm: MainThreadMarker) -> Self {
        let menu = Menu {
            quit_handler: QuitHandler::new(mtm),
        };
        menu.add_menus(mtm);
        menu
    }

    fn add_app_menu(&self, mtm: MainThreadMarker) -> Retained<NSMenu> {
        unsafe {
            let app_menu = NSMenu::new(mtm);
            let process_name = NSProcessInfo::processInfo().processName();
            let about_item = NSMenuItem::new(mtm);
            about_item.setTitle(&ns_string!("About ").stringByAppendingString(&process_name));
            about_item.setAction(Some(sel!(orderFrontStandardAboutPanel:)));
            app_menu.addItem(&about_item);

            let services_item = NSMenuItem::new(mtm);
            let services_menu = NSMenu::new(mtm);
            services_item.setTitle(ns_string!("Services"));
            services_item.setSubmenu(Some(&services_menu));
            app_menu.addItem(&services_item);

            let sep = NSMenuItem::separatorItem(mtm);
            app_menu.addItem(&sep);

            // application window operations
            let hide_item = NSMenuItem::new(mtm);
            hide_item.setTitle(&ns_string!("Hide ").stringByAppendingString(&process_name));
            hide_item.setKeyEquivalent(ns_string!("h"));
            hide_item.setAction(Some(sel!(hide:)));
            app_menu.addItem(&hide_item);

            let hide_others_item = NSMenuItem::new(mtm);
            hide_others_item.setTitle(ns_string!("Hide Others"));
            hide_others_item.setKeyEquivalent(ns_string!("h"));
            hide_others_item.setKeyEquivalentModifierMask(
                NSEventModifierFlags::NSEventModifierFlagOption
                    | NSEventModifierFlags::NSEventModifierFlagCommand,
            );
            hide_others_item.setAction(Some(sel!(hideOtherApplications:)));
            app_menu.addItem(&hide_others_item);

            let show_all_item = NSMenuItem::new(mtm);
            show_all_item.setTitle(ns_string!("Show All"));
            show_all_item.setAction(Some(sel!(unhideAllApplications:)));

            // quit
            let sep = NSMenuItem::separatorItem(mtm);
            app_menu.addItem(&sep);

            let quit_item = NSMenuItem::new(mtm);
            quit_item.setTitle(&ns_string!("Quit ").stringByAppendingString(&process_name));
            quit_item.setKeyEquivalent(ns_string!("q"));
            quit_item.setAction(Some(sel!(quit:)));
            quit_item.setTarget(Some(&self.quit_handler));
            app_menu.addItem(&quit_item);

            app_menu
        }
    }

    fn add_menus(&self, mtm: MainThreadMarker) {
        let app = NSApplication::sharedApplication(mtm);

        let main_menu = NSMenu::new(mtm);

        unsafe {
            let app_menu = self.add_app_menu(mtm);
            let app_menu_item = NSMenuItem::new(mtm);
            app_menu_item.setSubmenu(Some(&app_menu));
            if let Some(services_menu) = app_menu.itemWithTitle(ns_string!("Services")) {
                app.setServicesMenu(services_menu.submenu().as_deref());
            }
            main_menu.addItem(&app_menu_item);

            let file_menu = self.add_file_menu(mtm);
            let file_menu_item = NSMenuItem::new(mtm);
            file_menu_item.setSubmenu(Some(&file_menu));
            main_menu.addItem(&file_menu_item);

            let view_menu = self.add_view_menu(mtm);
            let view_menu_item = NSMenuItem::new(mtm);
            view_menu_item.setSubmenu(Some(&view_menu));
            main_menu.addItem(&view_menu_item);

            let tab_menu = self.add_tab_menu(mtm);
            let tab_menu_item = NSMenuItem::new(mtm);
            tab_menu_item.setSubmenu(Some(&tab_menu));
            main_menu.addItem(&tab_menu_item);

            let win_menu = self.add_window_menu(mtm);
            let win_menu_item = NSMenuItem::new(mtm);
            win_menu_item.setSubmenu(Some(&win_menu));
            main_menu.addItem(&win_menu_item);
            app.setWindowsMenu(Some(&win_menu));

            let help_menu = self.add_help_menu(mtm);
            let help_menu_item = NSMenuItem::new(mtm);
            help_menu_item.setSubmenu(Some(&help_menu));
            main_menu.addItem(&help_menu_item);
            app.setHelpMenu(Some(&help_menu));
        }
        app.setMainMenu(Some(&main_menu));
    }

    fn add_file_menu(&self, mtm: MainThreadMarker) -> Retained<NSMenu> {
        unsafe {
            let menu = NSMenu::new(mtm);
            menu.setTitle(ns_string!("File"));
            menu
        }
    }

    fn add_view_menu(&self, mtm: MainThreadMarker) -> Retained<NSMenu> {
        unsafe {
            let menu = NSMenu::new(mtm);
            menu.setTitle(ns_string!("View"));
            menu
        }
    }

    fn add_tab_menu(&self, mtm: MainThreadMarker) -> Retained<NSMenu> {
        unsafe {
            let menu = NSMenu::new(mtm);
            menu.setTitle(ns_string!("Tab"));
            menu
        }
    }

    fn add_help_menu(&self, mtm: MainThreadMarker) -> Retained<NSMenu> {
        unsafe {
            let menu = NSMenu::new(mtm);
            menu.setTitle(ns_string!("Help"));

            let separator_item = NSMenuItem::separatorItem(mtm);
            menu.addItem(&separator_item);

            let repository_item = NSMenuItem::new(mtm);
            repository_item.setTitle(ns_string!("HyperAST's Repository"));
            repository_item.setAction(Some(sel!(openRepository:)));
            menu.addItem(&repository_item);

            menu
        }
    }

    fn add_window_menu(&self, mtm: MainThreadMarker) -> Retained<NSMenu> {
        unsafe {
            let menu = NSMenu::new(mtm);
            menu.setTitle(ns_string!("Window"));

            let full_screen_item = NSMenuItem::new(mtm);
            full_screen_item.setTitle(ns_string!("Enter Full Screen"));
            full_screen_item.setKeyEquivalent(ns_string!("f"));
            full_screen_item.setAction(Some(sel!(toggleFullScreen:)));
            full_screen_item.setKeyEquivalentModifierMask(
                NSEventModifierFlags::NSEventModifierFlagControl
                    | NSEventModifierFlags::NSEventModifierFlagCommand,
            );
            menu.addItem(&full_screen_item);

            let min_item = NSMenuItem::new(mtm);
            min_item.setTitle(ns_string!("Minimize"));
            min_item.setKeyEquivalent(ns_string!("m"));
            min_item.setAction(Some(sel!(performMiniaturize:)));
            menu.addItem(&min_item);
            menu
        }
    }
}

#[derive(Clone)]
struct QuitHandlerIvars {}

declare_class!(
    #[derive(Debug)]
    struct QuitHandler;

    unsafe impl ClassType for QuitHandler {
        type Super = NSObject;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "QuitHandler";
    }

    impl DeclaredClass for QuitHandler {
        type Ivars = QuitHandlerIvars;
    }

    unsafe impl QuitHandler {
        #[method(quit:)]
        unsafe fn quit(&self, _event: &NSEvent) {
            std::process::exit(0)
        }
    }
);

impl QuitHandler {
    fn new(mtm: MainThreadMarker) -> Retained<QuitHandler> {
        unsafe { msg_send_id![mtm.alloc(), init] }
    }
}
