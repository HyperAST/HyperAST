use cocoa::{
    appkit::{NSApp, NSApplication, NSMenu, NSMenuItem, NSWindow},
    base::{id, nil, selector},
    foundation::{NSAutoreleasePool, NSInteger, NSString},
};
use objc::{
    class,
    declare::ClassDecl,
    msg_send,
    runtime::{Class, Object, Sel},
    sel, sel_impl,
};
use std::{ffi::c_void, fmt::Debug, ptr, sync::OnceLock};

const MAC_PLATFORM_IVAR: &str = "platform";
static ACTIONS: &[&str] = &["Alert", "Export"];
static MP: OnceLock<MacPlatform> = OnceLock::new();

pub(crate) struct MacPlatform(std::sync::Mutex<MacPlatformState>);

unsafe impl Send for MacPlatform {}
unsafe impl Sync for MacPlatform {}

pub(crate) struct MacPlatformState {
    menu_command: Option<Box<dyn FnMut(&dyn Action)>>,
    menu_actions: Vec<Box<dyn Action>>,
}

pub trait Action: 'static + Send + Debug {
    /// Clone the action into a new box
    fn boxed_clone(&self) -> Box<dyn Action>;

    /// Cast the action to the any type
    fn as_any(&self) -> &dyn std::any::Any;

    /// Do a partial equality check on this action and the other
    fn partial_eq(&self, action: &dyn Action) -> bool;

    /// Get the name of this action, for displaying in UI
    fn name(&self) -> &str;

    /// Get the name of this action for debugging
    fn debug_name() -> &'static str
    where
        Self: Sized;

    /// Build this action from a JSON value. This is used to construct actions from the keymap.
    /// A value of `{}` will be passed for actions that don't have any parameters.
    fn build(value: serde_json::Value) -> Result<Box<dyn Action>, ()>
    where
        Self: Sized;
}

/// A menu of the application, either a main menu or a submenu
#[derive(Debug)]
pub struct Menu<'a> {
    /// The name of the menu
    pub name: &'a str,

    /// The items in the menu
    pub items: Vec<MenuItem<'a>>,
}

#[derive(Debug)]
pub enum MenuItem<'a> {
    /// A separator between items
    Separator,

    /// A submenu
    Submenu(Menu<'a>),

    /// An action that can be performed
    Action {
        /// The name of this menu item
        name: &'a str,

        /// the action to perform when this menu item is selected
        action: Box<dyn Action>,
    },
}

pub(crate) fn show_nat_menu(_ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // nothing to do here
}

pub(crate) fn init_nat_menu() {
    return; // TODO issue with NSApplication. Start by trying: https://github.com/rust-windowing/winit/issues/4015
    let action_export = Box::new(Act::<1>);
    let action_alert = Box::new(Act::<0>);
    let mac_platform_state = MacPlatformState {
        menu_command: Some(Box::new(|c| {
            dbg!(c.name());
        })),
        menu_actions: vec![action_export.boxed_clone(), action_alert.boxed_clone()],
    };

    let mac_platform = MacPlatform(std::sync::Mutex::new(mac_platform_state));
    assert!(MP.set(mac_platform).is_ok());
    unsafe {
        let app = NSApp();
        static mut APP_DELEGATE_CLASS: *const Class = ptr::null();

        APP_DELEGATE_CLASS = {
            let mut decl = ClassDecl::new("GPUIApplicationDelegate", class!(NSResponder)).unwrap();
            decl.add_ivar::<*mut c_void>(MAC_PLATFORM_IVAR);

            decl.add_method(
                sel!(handleGPUIMenuItem:),
                handle_menu_item as extern "C" fn(&mut Object, Sel, id),
            );

            decl.register()
        };
        let menu = app.mainMenu();

        let app_delegate: id = msg_send![APP_DELEGATE_CLASS, new];
        app.setDelegate_(app_delegate);
        (*app_delegate).set_ivar(MAC_PLATFORM_IVAR, std::ptr::addr_of!(MP) as *const c_void);

        let delegate = app.delegate();
        let items = [
            MenuItem::Submenu(Menu {
                name: "File",
                items: vec![
                    MenuItem::Submenu(Menu {
                        name: "Save",
                        items: vec![],
                    }),
                    MenuItem::Action {
                        name: "Export",
                        action: action_export,
                    },
                    MenuItem::Submenu(Menu {
                        name: "Export As",
                        items: vec![],
                    }),
                    MenuItem::Action {
                        name: "Alert",
                        action: action_alert,
                    },
                ],
            }),
            MenuItem::Submenu(Menu {
                name: "Edit",
                items: vec![],
            }),
            MenuItem::Submenu(Menu {
                name: "Selection",
                items: vec![],
            }),
            MenuItem::Submenu(Menu {
                name: "View",
                items: vec![],
            }),
            MenuItem::Submenu(Menu {
                name: "Go",
                items: vec![],
            }),
            MenuItem::Submenu(Menu {
                name: "Help",
                items: vec![],
            }),
            MenuItem::Submenu(Menu {
                name: "EXPERIMENTAL!",
                items: vec![],
            }),
        ];
        let mut actions = vec![];
        for item_config in items {
            dbg!(&item_config);
            menu.addItem_(create_menu_item(item_config, delegate, &mut actions));
            dbg!(menu);
        }
    }

    unsafe fn create_menu_item(
        item: MenuItem<'_>,
        delegate: id,
        actions: &mut Vec<Box<dyn Action>>,
    ) -> id {
        match item {
            MenuItem::Separator => NSMenuItem::separatorItem(nil),
            MenuItem::Action { name, action } => {
                let item;
                let selector = selector("handleGPUIMenuItem:");
                item = NSMenuItem::alloc(nil)
                    .initWithTitle_action_keyEquivalent_(ns_string(name), selector, ns_string(""))
                    .autorelease();
                // let tag = actions.len() as NSInteger;
                let tag = MP
                    .get()
                    .unwrap()
                    .0
                    .lock()
                    .unwrap()
                    .menu_actions
                    .iter()
                    .position(|x| x.partial_eq(&*action))
                    .unwrap();
                let tag = tag as NSInteger;
                let _: () = msg_send![item, setTag: tag];
                actions.push(action);
                item
            }
            MenuItem::Submenu(Menu { name, items }) => {
                let item = NSMenuItem::new(nil).autorelease();
                let submenu = NSMenu::new(nil).autorelease();
                submenu.setDelegate_(delegate);
                for item in items {
                    submenu.addItem_(create_menu_item(item, delegate, actions));
                }
                item.setSubmenu_(submenu);
                item.setTitle_(ns_string(name));
                item
            }
        }
    }

    unsafe fn ns_string(string: &str) -> id {
        NSString::alloc(nil).init_str(string).autorelease()
    }
}

#[derive(Debug, Clone)]
struct Act<const N: usize>;

impl<const N: usize> Action for Act<N> {
    fn boxed_clone(&self) -> Box<dyn Action> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn partial_eq(&self, action: &dyn Action) -> bool {
        action.as_any().is::<Act<N>>()
    }

    fn name(&self) -> &str {
        ACTIONS[N]
    }

    fn debug_name() -> &'static str
    where
        Self: Sized,
    {
        ACTIONS[N]
    }

    fn build(value: serde_json::Value) -> Result<Box<dyn Action>, ()>
    where
        Self: Sized,
    {
        let s: Box<dyn Action> = Box::new(Self);
        value
            .as_str()
            .ok_or(())?
            .eq(ACTIONS[N])
            .then_some(s)
            .ok_or(())
    }
}

extern "C" fn handle_menu_item(this: &mut Object, _: Sel, item: id) {
    unsafe {
        let platform = get_mac_platform(this);
        let mut lock = platform.0.lock().unwrap();
        if let Some(mut callback) = lock.menu_command.take() {
            let tag: NSInteger = msg_send![item, tag];
            let index = tag as usize;
            if let Some(action) = lock.menu_actions.get(index) {
                let action = action.boxed_clone();
                drop(lock);
                callback(&*action);
            }
            platform
                .0
                .lock()
                .unwrap()
                .menu_command
                .get_or_insert(callback);
        }
    }
}
unsafe fn get_mac_platform(_object: &mut Object) -> &MacPlatform {
    MP.get().unwrap()
}
