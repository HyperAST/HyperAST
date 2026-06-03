//! inital version from https://github.com/rerun-io/rerun/blob/04028f6bc2c78d5eb40036325cc6addb67550b52/crates/viewer/re_ui/src/command.rs

use egui::{Key, KeyboardShortcut, Modifiers};

/// Interface for sending [`UICommand`] messages.
pub trait UICommandSender {
    fn send_ui(&self, command: UICommand);
}

/// All the commands we support.
///
/// Most are available in the GUI,
/// some have keyboard shortcuts,
/// and all are visible in the [`crate::CommandPalette`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, strum_macros::EnumIter)]
pub enum UICommand {
    // Listed in the order they show up in the command palette by default!
    Open,
    SaveResults,
    SaveLayout,
    ResetLayout,
    // kind of temporary, because of issues with persistance in browser on close
    PersistApp,
    // SaveRecording,
    // SaveRecordingSelection,
    SaveBlueprint,
    CloseCurrentRecording,
    CloseAllRecordings,

    #[cfg(not(target_arch = "wasm32"))]
    Quit,

    OpenWebHelp,
    // OpenRerunDiscord,
    ResetViewer,
    ClearAndGenerateBlueprint,

    #[cfg(not(target_arch = "wasm32"))]
    OpenProfiler,

    TogglePanelStateOverrides,
    ToggleMemoryPanel,
    ToggleTopPanel,
    ToggleBlueprintPanel,
    ToggleSelectionPanel,
    ToggleTimePanel,

    #[cfg(debug_assertions)]
    ToggleBlueprintInspectionPanel,

    #[cfg(debug_assertions)]
    ToggleEguiDebugPanel,

    ToggleFullscreen,
    #[cfg(not(target_arch = "wasm32"))]
    ZoomIn,
    #[cfg(not(target_arch = "wasm32"))]
    ZoomOut,
    #[cfg(not(target_arch = "wasm32"))]
    ZoomReset,

    SelectionPrevious,
    SelectionNext,

    ToggleCommandPalette,

    // Playback:
    PlaybackTogglePlayPause,
    PlaybackFollow,
    PlaybackStepBack,
    PlaybackStepForward,
    PlaybackRestart,

    // Dev-tools:
    #[cfg(not(target_arch = "wasm32"))]
    ScreenshotWholeApp,
    #[cfg(not(target_arch = "wasm32"))]
    PrintDataStore,
    #[cfg(not(target_arch = "wasm32"))]
    PrintBlueprintStore,
    #[cfg(not(target_arch = "wasm32"))]
    ClearPrimaryCache,
    #[cfg(not(target_arch = "wasm32"))]
    PrintPrimaryCache,

    #[cfg(target_arch = "wasm32")]
    CopyDirectLink,

    // Graphics options:
    #[cfg(target_arch = "wasm32")]
    RestartWithWebGl,
    #[cfg(target_arch = "wasm32")]
    RestartWithWebGpu,

    // NOTE: could take inspiration from zed on (kb) interations
    NewQuery,

    // Compute commands:
    RunQuery,
    ComputeTrackingMappingFuture,
    ComputeTrackingMappingPast,
    ComputeMappingFuture,
    ComputeMappingPast,
    ComputeDiffFuture,
    ComputeDiffPast,
    FindAllReferences,
    FindDeclaration,

    // Navigation commands:
    // do not always need a compute on the backend, they can often do a more focussed computation
    /// NOTE: do not need to compute anything if [`UICommand::FindDeclaration`] was ran on current commit.
    GotoDeclaration,
    /// Cannot directly goto references
    /// NOTE: do not need to compute anything if [`UICommand::FindAllReferences`] was ran on current commit.
    ShowReferences,
    /// NOTE: do not need to compute anything if [`UICommand::ComputeTrackingMappingFuture`] was ran on current commit.
    TrackFuture,
    /// can be conceptually mapped to `git blame`
    /// NOTE: do not need to compute anything if [`UICommand::ComputeTrackingMappingPast`] was ran on current commit.
    TrackPast,
    ShowCommitMetadata,
}

impl UICommand {
    pub fn text(self) -> &'static str {
        self.text_and_tooltip().0
    }

    pub fn tooltip(self) -> &'static str {
        self.text_and_tooltip().1
    }

    pub fn text_and_tooltip(self) -> (&'static str, &'static str) {
        match self {
            Self::SaveResults => ("Save Results…", "Save all results and associated config"),
            Self::SaveLayout => ("Save Current Layout…", "Save current layout"),
            Self::ResetLayout => (
                "Reset Current Layout…",
                "Reset current layout to last recorded state",
            ),
            Self::PersistApp => (
                "Persist App…",
                "Enable persistance and save persited state to storage",
            ),
            // Self::SaveRecording => ("Save recording…", "Save all data to a Rerun data file (.rrd)"),

            // Self::SaveRecordingSelection => (
            //     "Save current time selection…",
            //     "Save data for the current loop selection to a Rerun data file (.rrd)",
            // ),
            Self::SaveBlueprint => (
                "Save blueprint…",
                "Save the current viewer setup as a Rerun blueprint file (.rbl)",
            ),

            Self::Open => (
                "Open…",
                "Open any supported files (.rrd, images, meshes, …)",
            ),

            Self::CloseCurrentRecording => (
                "Close current recording",
                "Close the current recording (unsaved data will be lost)",
            ),

            Self::CloseAllRecordings => (
                "Close all recordings",
                "Close all open current recording (unsaved data will be lost)",
            ),

            #[cfg(not(target_arch = "wasm32"))]
            Self::Quit => ("Quit", "Close the Rerun Viewer"),

            Self::OpenWebHelp => (
                "Help",
                "Visit the help page on our website, with troubleshooting tips and more",
            ),
            // Self::OpenRerunDiscord => ("Rerun Discord", "Visit the Rerun Discord server, where you can ask questions and get help"),
            Self::ResetViewer => (
                "Reset Viewer",
                "Reset the Viewer to how it looked the first time you ran it, forgetting all stored blueprints and UI state",
            ),

            Self::ClearAndGenerateBlueprint => (
                "Clear and generate new blueprint",
                "Clear the current blueprint and generate a new one based on heuristics.",
            ),

            #[cfg(not(target_arch = "wasm32"))]
            Self::OpenProfiler => (
                "Open profiler",
                "Starts a profiler, showing what makes the viewer run slow",
            ),

            Self::ToggleMemoryPanel => (
                "Toggle memory panel",
                "View and track current RAM usage inside Rerun Viewer",
            ),

            Self::TogglePanelStateOverrides => (
                "Toggle panel state overrides",
                "Toggle panel state between app blueprint and overrides",
            ),
            Self::ToggleTopPanel => ("Toggle top panel", "Toggle the top panel"),
            Self::ToggleBlueprintPanel => ("Toggle blueprint panel", "Toggle the left panel"),
            Self::ToggleSelectionPanel => ("Toggle selection panel", "Toggle the right panel"),
            Self::ToggleTimePanel => ("Toggle time panel", "Toggle the bottom panel"),

            #[cfg(debug_assertions)]
            Self::ToggleBlueprintInspectionPanel => (
                "Toggle blueprint inspection panel",
                "Inspect the timeline of the internal blueprint data.",
            ),

            #[cfg(debug_assertions)]
            Self::ToggleEguiDebugPanel => (
                "Toggle egui debug panel",
                "View and change global egui style settings",
            ),

            #[cfg(not(target_arch = "wasm32"))]
            Self::ToggleFullscreen => (
                "Toggle fullscreen",
                "Toggle between windowed and fullscreen viewer",
            ),

            #[cfg(target_arch = "wasm32")]
            Self::ToggleFullscreen => (
                "Toggle fullscreen",
                "Toggle between full viewport dimensions and initial dimensions",
            ),

            #[cfg(not(target_arch = "wasm32"))]
            Self::ZoomIn => ("Zoom in", "Increases the UI zoom level"),
            #[cfg(not(target_arch = "wasm32"))]
            Self::ZoomOut => ("Zoom out", "Decreases the UI zoom level"),
            #[cfg(not(target_arch = "wasm32"))]
            Self::ZoomReset => (
                "Reset zoom",
                "Resets the UI zoom level to the operating system's default value",
            ),

            Self::SelectionPrevious => ("Previous selection", "Go to previous selection"),
            Self::SelectionNext => ("Next selection", "Go to next selection"),
            Self::ToggleCommandPalette => ("Command palette…", "Toggle the Command Palette"),

            Self::PlaybackTogglePlayPause => ("Toggle play/pause", "Either play or pause the time"),
            Self::PlaybackFollow => ("Follow", "Follow on from end of timeline"),
            Self::PlaybackStepBack => (
                "Step time back",
                "Move the time marker back to the previous point in time with any data",
            ),
            Self::PlaybackStepForward => (
                "Step time forward",
                "Move the time marker to the next point in time with any data",
            ),
            Self::PlaybackRestart => ("Restart", "Restart from beginning of timeline"),

            #[cfg(not(target_arch = "wasm32"))]
            Self::ScreenshotWholeApp => (
                "Screenshot",
                "Copy screenshot of the whole app to clipboard",
            ),
            #[cfg(not(target_arch = "wasm32"))]
            Self::PrintDataStore => (
                "Print datastore",
                "Prints the entire data store to the console and clipboard. WARNING: this may be A LOT of text.",
            ),
            #[cfg(not(target_arch = "wasm32"))]
            Self::PrintBlueprintStore => (
                "Print blueprint store",
                "Prints the entire blueprint store to the console and clipboard. WARNING: this may be A LOT of text.",
            ),
            #[cfg(not(target_arch = "wasm32"))]
            Self::ClearPrimaryCache => (
                "Clear primary cache",
                "Clears the primary cache in its entirety.",
            ),
            #[cfg(not(target_arch = "wasm32"))]
            Self::PrintPrimaryCache => (
                "Print primary cache",
                "Prints the state of the entire primary cache to the console and clipboard. WARNING: this may be A LOT of text.",
            ),

            #[cfg(target_arch = "wasm32")]
            Self::CopyDirectLink => (
                "Copy direct link",
                "Copy a link to the viewer with the URL parameter set to the current .rrd data source.",
            ),

            #[cfg(target_arch = "wasm32")]
            Self::RestartWithWebGl => (
                "Restart with WebGL",
                "Reloads the webpage and force WebGL for rendering. All data will be lost.",
            ),
            #[cfg(target_arch = "wasm32")]
            Self::RestartWithWebGpu => (
                "Restart with WebGPU",
                "Reloads the webpage and force WebGPU for rendering. All data will be lost.",
            ),

            UICommand::NewQuery => ("Create new query", "Create a new tree-sitter query"),

            UICommand::RunQuery => ("Run current code query", "TODO desc. RunQuery"),
            UICommand::ComputeTrackingMappingFuture => (
                "ComputeTrackingMappingFuture",
                "TODO desc. ComputeTrackingMappingFuture",
            ),
            UICommand::ComputeTrackingMappingPast => (
                "ComputeTrackingMappingPast",
                "TODO desc. ComputeTrackingMappingPast",
            ),
            UICommand::ComputeMappingFuture => {
                ("ComputeMappingFuture", "TODO desc. ComputeMappingFuture")
            }
            UICommand::ComputeMappingPast => {
                ("ComputeMappingPast", "TODO desc. ComputeMappingPast")
            }
            UICommand::ComputeDiffFuture => ("ComputeDiffFuture", "TODO desc. ComputeDiffFuture"),
            UICommand::ComputeDiffPast => ("ComputeDiffPast", "TODO desc. ComputeDiffPast"),
            UICommand::FindAllReferences => ("FindAllReferences", "TODO desc. FindAllReferences"),
            UICommand::FindDeclaration => ("FindDeclaration", "TODO desc. FindDeclaration"),
            UICommand::GotoDeclaration => ("GotoDeclaration", "TODO desc. GotoDeclaration"),
            UICommand::ShowReferences => ("ShowReferences", "TODO desc. ShowReferences"),
            UICommand::TrackFuture => ("TrackFuture", "TODO desc. TrackFuture"),
            UICommand::TrackPast => ("TrackPast", "TODO desc. TrackPast"),
            UICommand::ShowCommitMetadata => {
                ("ShowCommitMetadata", "TODO desc. ShowCommitMetadata")
            }
        }
    }

    #[allow(clippy::unnecessary_wraps)] // Only on some platforms
    pub fn kb_shortcut(self) -> Option<KeyboardShortcut> {
        fn key(key: Key) -> KeyboardShortcut {
            KeyboardShortcut::new(Modifiers::NONE, key)
        }

        fn cmd(key: Key) -> KeyboardShortcut {
            KeyboardShortcut::new(Modifiers::COMMAND, key)
        }

        fn cmd_alt(key: Key) -> KeyboardShortcut {
            KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::ALT), key)
        }

        fn ctrl_shift(key: Key) -> KeyboardShortcut {
            KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), key)
        }

        match self {
            Self::SaveResults => Some(cmd(Key::S)),
            Self::SaveLayout => Some(cmd_alt(Key::S)),
            Self::ResetLayout => None,
            Self::PersistApp => None,
            // Self::SaveRecording => Some(cmd(Key::S)),
            // Self::SaveRecordingSelection => Some(cmd_alt(Key::S)),
            Self::SaveBlueprint => None,
            Self::Open => Some(cmd(Key::O)),
            Self::CloseCurrentRecording => None,
            Self::CloseAllRecordings => None,

            #[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
            Self::Quit => Some(KeyboardShortcut::new(Modifiers::ALT, Key::F4)),

            Self::OpenWebHelp => None,
            // Self::OpenRerunDiscord => None,
            #[cfg(all(not(target_arch = "wasm32"), not(target_os = "windows")))]
            Self::Quit => Some(cmd(Key::Q)),

            Self::ResetViewer => Some(ctrl_shift(Key::R)),
            Self::ClearAndGenerateBlueprint => None,

            #[cfg(not(target_arch = "wasm32"))]
            Self::OpenProfiler => Some(ctrl_shift(Key::P)),
            Self::ToggleMemoryPanel => Some(ctrl_shift(Key::M)),
            Self::TogglePanelStateOverrides => None,
            Self::ToggleTopPanel => None,
            Self::ToggleBlueprintPanel => Some(ctrl_shift(Key::B)),
            Self::ToggleSelectionPanel => Some(ctrl_shift(Key::S)),
            Self::ToggleTimePanel => Some(ctrl_shift(Key::T)),

            #[cfg(debug_assertions)]
            Self::ToggleBlueprintInspectionPanel => Some(ctrl_shift(Key::I)),

            #[cfg(debug_assertions)]
            Self::ToggleEguiDebugPanel => Some(ctrl_shift(Key::U)),

            #[cfg(not(target_arch = "wasm32"))]
            Self::ToggleFullscreen => Some(key(Key::F11)),
            #[cfg(target_arch = "wasm32")]
            Self::ToggleFullscreen => None,

            #[cfg(not(target_arch = "wasm32"))]
            Self::ZoomIn => Some(egui::gui_zoom::kb_shortcuts::ZOOM_IN),
            #[cfg(not(target_arch = "wasm32"))]
            Self::ZoomOut => Some(egui::gui_zoom::kb_shortcuts::ZOOM_OUT),
            #[cfg(not(target_arch = "wasm32"))]
            Self::ZoomReset => Some(egui::gui_zoom::kb_shortcuts::ZOOM_RESET),

            Self::SelectionPrevious => Some(ctrl_shift(Key::ArrowLeft)),
            Self::SelectionNext => Some(ctrl_shift(Key::ArrowRight)),
            Self::ToggleCommandPalette => Some(cmd(Key::P)),

            Self::PlaybackTogglePlayPause => Some(key(Key::Space)),
            Self::PlaybackFollow => Some(cmd(Key::ArrowRight)),
            Self::PlaybackStepBack => Some(key(Key::ArrowLeft)),
            Self::PlaybackStepForward => Some(key(Key::ArrowRight)),
            Self::PlaybackRestart => Some(cmd(Key::ArrowLeft)),

            #[cfg(not(target_arch = "wasm32"))]
            Self::ScreenshotWholeApp => None,
            #[cfg(not(target_arch = "wasm32"))]
            Self::PrintDataStore => None,
            #[cfg(not(target_arch = "wasm32"))]
            Self::PrintBlueprintStore => None,
            #[cfg(not(target_arch = "wasm32"))]
            Self::ClearPrimaryCache => None,
            #[cfg(not(target_arch = "wasm32"))]
            Self::PrintPrimaryCache => None,

            #[cfg(target_arch = "wasm32")]
            Self::CopyDirectLink => None,

            #[cfg(target_arch = "wasm32")]
            Self::RestartWithWebGl => None,
            #[cfg(target_arch = "wasm32")]
            Self::RestartWithWebGpu => None,

            // TODO
            UICommand::NewQuery => None,
            UICommand::RunQuery => None,
            UICommand::ComputeTrackingMappingFuture => None,
            UICommand::ComputeTrackingMappingPast => None,
            UICommand::ComputeMappingFuture => None,
            UICommand::ComputeMappingPast => None,
            UICommand::ComputeDiffFuture => None,
            UICommand::ComputeDiffPast => None,
            UICommand::FindAllReferences => None,
            UICommand::FindDeclaration => None,
            UICommand::GotoDeclaration => None,
            UICommand::ShowReferences => None,
            UICommand::TrackFuture => None,
            UICommand::TrackPast => None,
            UICommand::ShowCommitMetadata => None,
        }
    }

    pub fn icon(self) -> Option<&'static re_ui::Icon> {
        match self {
            Self::OpenWebHelp => Some(&re_ui::icons::EXTERNAL_LINK),
            // Self::OpenRerunDiscord => Some(&re_ui::icons::DISCORD),
            _ => None,
        }
    }

    pub fn is_link(self) -> bool {
        matches!(
            self,
            Self::OpenWebHelp // | Self::OpenRerunDiscord
        )
    }

    #[must_use = "Returns the Command that was triggered by some keyboard shortcut"]
    pub fn listen_for_kb_shortcut(egui_ctx: &egui::Context) -> Option<Self> {
        use strum::IntoEnumIterator as _;

        let anything_has_focus = egui_ctx.memory(|mem| mem.focused().is_some());
        if anything_has_focus {
            return None; // e.g. we're typing in a TextField
        }

        let mut commands: Vec<(KeyboardShortcut, Self)> = Self::iter()
            .filter_map(|cmd| cmd.kb_shortcut().map(|kb_shortcut| (kb_shortcut, cmd)))
            .collect();

        // If the user pressed `Cmd-Shift-S` then egui will match that
        // with both `Cmd-Shift-S` and `Cmd-S`.
        // The reason is that `Shift` (and `Alt`) are sometimes required to produce certain keys,
        // such as `+` (`Shift =` on an american keyboard).
        // The result of this is that we bust check for `Cmd-Shift-S` before `Cmd-S`, etc.
        // So we order the commands here so that the commands with `Shift` and `Alt` in them
        // are checked first.
        commands.sort_by_key(|(kb_shortcut, _cmd)| {
            let num_shift_alts =
                kb_shortcut.modifiers.shift as i32 + kb_shortcut.modifiers.alt as i32;
            -num_shift_alts // most first
        });

        egui_ctx.input_mut(|input| {
            for (kb_shortcut, command) in commands {
                if input.consume_shortcut(&kb_shortcut) {
                    return Some(command);
                }
            }
            None
        })
    }

    /// Show this command as a menu-button.
    ///
    /// If clicked, enqueue the command.
    pub fn menu_button_ui(
        self,
        ui: &mut egui::Ui,
        command_sender: &impl UICommandSender,
    ) -> egui::Response {
        let button = self.menu_button(ui.ctx());
        let mut response = ui.add(button).on_hover_text(self.tooltip());

        if self.is_link() {
            response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
        }

        if response.clicked() {
            command_sender.send_ui(self);
            ui.close_menu();
        }

        response
    }

    pub fn menu_button(self, egui_ctx: &egui::Context) -> egui::Button<'static> {
        let mut button = if let Some(icon) = self.icon() {
            egui::Button::image_and_text(
                icon.as_image()
                    .fit_to_exact_size(re_ui::DesignTokens::small_icon_size()),
                self.text(),
            )
        } else {
            egui::Button::new(self.text())
        };

        if let Some(shortcut) = self.kb_shortcut() {
            button = button.shortcut_text(egui_ctx.format_shortcut(&shortcut));
        }

        button
    }

    /// Add e.g. " (Ctrl+F11)" as a suffix
    pub fn format_shortcut_tooltip_suffix(self, egui_ctx: &egui::Context) -> String {
        if let Some(kb_shortcut) = self.kb_shortcut() {
            format!(" ({})", egui_ctx.format_shortcut(&kb_shortcut))
        } else {
            Default::default()
        }
    }

    pub fn tooltip_with_shortcut(self, egui_ctx: &egui::Context) -> String {
        format!(
            "{}{}",
            self.tooltip(),
            self.format_shortcut_tooltip_suffix(egui_ctx)
        )
    }
}

#[test]
fn check_for_clashing_command_shortcuts() {
    fn clashes(a: KeyboardShortcut, b: KeyboardShortcut) -> bool {
        if a.logical_key != b.logical_key {
            return false;
        }

        if a.modifiers.alt != b.modifiers.alt {
            return false;
        }

        if a.modifiers.shift != b.modifiers.shift {
            return false;
        }

        // On Non-Mac, command is interpreted as ctrl!
        (a.modifiers.command || a.modifiers.ctrl) == (b.modifiers.command || b.modifiers.ctrl)
    }

    use strum::IntoEnumIterator as _;

    for a_cmd in UICommand::iter() {
        if let Some(a_shortcut) = a_cmd.kb_shortcut() {
            for b_cmd in UICommand::iter() {
                if a_cmd == b_cmd {
                    continue;
                }
                if let Some(b_shortcut) = b_cmd.kb_shortcut() {
                    assert!(
                        !clashes(a_shortcut, b_shortcut),
                        "Command '{a_cmd:?}' and '{b_cmd:?}' have overlapping keyboard shortcuts: {:?} vs {:?}",
                        a_shortcut.format(&egui::ModifierNames::NAMES, true),
                        b_shortcut.format(&egui::ModifierNames::NAMES, true),
                    );
                }
            }
        }
    }
}

/// Sender that queues up the execution of a command.
pub struct CommandSender(std::sync::mpsc::Sender<crate::command::UICommand>);

/// Receiver for the [`CommandSender`]
pub struct CommandReceiver(std::sync::mpsc::Receiver<crate::command::UICommand>);

/// Creates a new command channel.
pub fn command_channel() -> (CommandSender, CommandReceiver) {
    let (sender, receiver) = std::sync::mpsc::channel();
    (CommandSender(sender), CommandReceiver(receiver))
}

impl crate::command::UICommandSender for CommandSender {
    /// Send a command to be executed.
    fn send_ui(&self, command: UICommand) {
        // The only way this can fail is if the receiver has been dropped.
        self.0.send(command).ok();
    }
}

impl CommandReceiver {
    /// Receive a command to be executed if any is queued.
    pub fn recv(&self) -> Option<UICommand> {
        // The only way this can fail (other than being empty)
        // is if the sender has been dropped.
        self.0.try_recv().ok()
    }
}
