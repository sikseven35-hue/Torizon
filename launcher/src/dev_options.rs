//! Developer options: a Settings section nothing in the window points to. Seven clicks on the
//! sidebar's "Launcher x.y.z" reveal it for this run; it stays shown while one of its options is
//! on. Its one option lets offline names play without an account that owns the game
//! (`backend::Settings::dev_offline`). The game still comes from the player's own install: the
//! launcher downloads Mojang's files only for an owner (`backend::download_for_owner`).
use super::*;

/// Clicks on the launcher's version that reveal the section.
const CLICKS: u8 = 7;

impl Launcher {
    /// A click on the sidebar's launcher version; the last one opens Settings on the section.
    pub(super) fn version_clicked(&mut self) {
        if self.dev_options {
            return;
        }
        self.dev_clicks += 1;
        if self.dev_clicks >= CLICKS {
            self.dev_options = true;
            self.go(Page::Settings);
        }
    }

    pub(super) fn dev_options_section(&mut self, cx: &mut Context<Self>) -> Option<Div> {
        Some(self.torizon_options(cx))
    }
}
