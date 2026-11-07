//! First-launch preferences and optional animated mascots for Torizon.
use super::*;
use crate::servers_page::Field;
use std::sync::{Arc, LazyLock};

#[derive(Default)]
pub(super) struct Setup {
    initialized: bool,
    greeted: bool,
    step: usize,
    profile: usize,
    error: Option<String>,
}

static WAVE: LazyLock<Arc<gpui::Image>> = LazyLock::new(|| {
    Arc::new(gpui::Image::from_bytes(
        gpui::ImageFormat::Gif,
        include_bytes!("../assets/torizon-cats/wave.gif").to_vec(),
    ))
});
static WINK: LazyLock<Arc<gpui::Image>> = LazyLock::new(|| {
    Arc::new(gpui::Image::from_bytes(
        gpui::ImageFormat::Gif,
        include_bytes!("../assets/torizon-cats/wink.gif").to_vec(),
    ))
});
static WAVE_STILL: LazyLock<Arc<gpui::Image>> = LazyLock::new(|| {
    Arc::new(gpui::Image::from_bytes(
        gpui::ImageFormat::Png,
        include_bytes!("../assets/torizon-cats/wave.png").to_vec(),
    ))
});

static WINK_STILL: LazyLock<Arc<gpui::Image>> = LazyLock::new(|| {
    Arc::new(gpui::Image::from_bytes(
        gpui::ImageFormat::Png,
        include_bytes!("../assets/torizon-cats/wink.png").to_vec(),
    ))
});

const TITLES: [&str; 4] = [
    "Welcome to Torizon",
    "Your Minecraft files",
    "Performance first",
    "Make it yours",
];

/// Update one option without discarding unrelated settings or player data.
fn option(path: &std::path::Path, key: &str, value: &str) -> std::io::Result<()> {
    let prefix = format!("{key}:");
    let previous = std::fs::read_to_string(path).unwrap_or_default();
    let mut lines: Vec<String> = previous
        .lines()
        .filter(|line| !line.starts_with(&prefix))
        .map(str::to_owned)
        .collect();
    lines.push(format!("{key}:{value}"));
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, lines.join("\n") + "\n")
}

impl Launcher {
    pub(super) fn torizon_mascot(&self, size: f32, alternate: bool) -> Div {
        let image = if self.client.is_some() {
            if alternate {
                WINK_STILL.clone()
            } else {
                WAVE_STILL.clone()
            }
        } else if alternate {
            WINK.clone()
        } else {
            WAVE.clone()
        };
        div()
            .flex_none()
            .w(px(size))
            .h(px(size * if alternate { 373. / 498. } else { 374. / 498. }))
            .overflow_hidden()
            .child(
                gpui::img(image)
                    .id(if alternate {
                        "mascot-wink"
                    } else {
                        "mascot-wave"
                    })
                    .size_full(),
            )
    }

    pub(super) fn setup_advance(&mut self) {
        self.torizon_setup.error = None;
        if !self.torizon_setup.greeted {
            self.torizon_setup.greeted = true;
            return;
        }
        if self.torizon_setup.step == 0 {
            let name = self.servers.value(Field::SetupName).trim().to_string();
            if !account::valid_offline_name(&name) {
                self.torizon_setup.error =
                    Some(tr("Use 3 to 16 letters, digits or underscores.").into());
                return;
            }
            self.play_as(account::Player::Offline(name));
        }
        if self.torizon_setup.step == 1 {
            let path = self
                .servers
                .value(Field::SetupPath)
                .trim()
                .trim_matches('"');
            if !path.is_empty() {
                let path = std::path::PathBuf::from(path);
                if !path.is_dir() {
                    self.torizon_setup.error =
                        Some(tr("Choose an existing Minecraft folder.").into());
                    return;
                }
                self.settings.minecraft_dir = Some(path);
            }
        }
        if self.torizon_setup.step < 3 {
            self.torizon_setup.step += 1;
            return;
        }
        let (distance, simulation, fps) = match self.torizon_setup.profile {
            0 => (8, 6, 144),
            2 => (20, 10, 144),
            _ => (12, 8, 144),
        };
        let home = backend::game_home();
        let result = (|| -> std::io::Result<()> {
            for (key, value) in [
                ("renderDistance", distance),
                ("simulationDistance", simulation),
                ("maxFps", fps),
            ] {
                option(&home.join("options.txt"), key, &value.to_string())?;
            }
            option(&home.join("options.txt"), "enableVsync", "false")?;
            let language = match self.settings.language() {
                i18n::Language::Russian => "ru_ru",
                i18n::Language::Ukrainian => "uk_ua",
                _ => "en_us",
            };
            option(&home.join("options.txt"), "lang", language)?;
            option(&home.join("horizon-graphics.txt"), "workers", "0")?;
            Ok(())
        })();
        if let Err(error) = result {
            self.torizon_setup.error = Some(error.to_string());
            return;
        }
        self.settings.torizon_setup_done = true;
        if !self.save_settings() {
            self.settings.torizon_setup_done = false;
            self.torizon_setup.error = self.settings_save_error.clone();
        }
    }

    fn setup_choose_folder(&mut self, cx: &mut Context<Self>) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(tr("Choose").into()),
        });
        cx.spawn(async move |this, cx| {
            if let Ok(Ok(Some(mut paths))) = paths.await {
                if let Some(path) = paths.pop() {
                    let _ = this.update(cx, |this, cx| {
                        this.servers
                            .text(Field::SetupPath)
                            .set(path.display().to_string());
                        cx.notify();
                    });
                }
            }
        })
        .detach();
    }

    pub(super) fn torizon_setup_screen(&mut self, cx: &mut Context<Self>) -> Option<Stateful<Div>> {
        if self.settings.torizon_setup_done {
            return None;
        }
        if !self.torizon_setup.initialized {
            let name = self.who_plays().0;
            self.servers.text(Field::SetupName).set(name);
            let extra = std::env::var_os("APPDATA")
                .map(std::path::PathBuf::from)
                .or_else(dirs::data_dir)
                .map(|dir| dir.join(".tlauncher/legacy/Minecraft/game"));
            let path = self
                .settings
                .minecraft_dir
                .clone()
                .or_else(|| extra.filter(|path| path.is_dir()))
                .or_else(backend::default_minecraft_dir);
            self.servers
                .text(Field::SetupPath)
                .set(path.map(|p| p.display().to_string()).unwrap_or_default());
            self.torizon_setup.profile = 1;
            self.torizon_setup.initialized = true;
        }
        let t = self.theme;
        let step = self.torizon_setup.step;
        if !self.torizon_setup.greeted {
            return Some(
                div()
                    .id("torizon-welcome")
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        glass::plate(24.)
                            .w(px(860.))
                            .p(px(36.))
                            .border_1()
                            .border_color(t.rule)
                            .flex()
                            .items_center()
                            .gap(px(24.))
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(20.))
                                    .child(text(theme::TITLE).child(tr("Welcome to Torizon")))
                                    .child(
                                        text(theme::UI)
                                            .text_color(t.ink_soft)
                                            .child(tr("Choose your language and offline name.")),
                                    )
                                    .child(
                                        button(
                                            "welcome-start",
                                            "Let's set things up",
                                            Kind::Primary,
                                            &t,
                                        )
                                        .on_click(
                                            cx.listener(|this, _: &ClickEvent, _, cx| {
                                                this.setup_advance();
                                                cx.notify();
                                            }),
                                        ),
                                    ),
                            )
                            .child(self.torizon_mascot(330., false))
                            .relative()
                            .with_animation("welcome-enter", motion::eased(motion::POP), |d, p| {
                                d.opacity(p).top(px(motion::POP_RISE * (1. - p)))
                            }),
                    ),
            );
        }
        let mut body = div().flex().flex_col().gap(px(16.)).min_h(px(240.));
        match step {
            0 => {
                let mut languages = div().flex().gap(px(8.));
                for (id, label, language) in [
                    ("setup-ru", "Русский", i18n::Language::Russian),
                    ("setup-en", "English", i18n::Language::English),
                    ("setup-uk", "Українська", i18n::Language::Ukrainian),
                ] {
                    languages = languages.child(button(id, label, Kind::Secondary, &t).on_click(
                        cx.listener(move |this, _: &ClickEvent, _, cx| {
                            this.settings.language = Some(language);
                            i18n::set(language);
                            cx.notify();
                        }),
                    ));
                }
                body = body
                    .child(text(theme::UI).child(tr("Choose your language and offline name.")))
                    .child(languages)
                    .child(self.field(Field::SetupName, "Player name", 430., cx))
                    .child(
                        text(theme::CAPTION)
                            .text_color(t.muted)
                            .child(tr("No Microsoft sign-in is needed for offline play.")),
                    );
            }
            1 => {
                body = body.child(text(theme::UI).child(tr("Choose the Minecraft 26.3 folder that contains versions and assets.")))
                    .child(self.field(Field::SetupPath, "Minecraft folder", 430., cx))
                    .child(button("setup-browse", "Choose folder", Kind::Secondary, &t)
                        .on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.setup_choose_folder(cx))))
                    .child(text(theme::CAPTION).text_color(t.muted).child(tr("Fabric and TLauncher installations are supported. Your existing worlds stay in place.")));
            }
            2 => {
                body = body.child(text(theme::UI).child(tr(
                    "Choose a starting profile. You can change everything later in the game.",
                )));
                for (index, name, note) in [
                    (0, "Smooth", "8 chunks, simulation 6, 144 FPS limit"),
                    (1, "Balanced", "12 chunks, simulation 8, 144 FPS limit"),
                    (2, "Detailed", "20 chunks, simulation 10, 144 FPS limit"),
                ] {
                    let chosen = self.torizon_setup.profile == index;
                    body = body
                        .child(
                            button(
                                SharedString::from(format!("setup-profile-{index}")),
                                name,
                                if chosen {
                                    Kind::Primary
                                } else {
                                    Kind::Secondary
                                },
                                &t,
                            )
                            .on_click(cx.listener(
                                move |this, _: &ClickEvent, _, cx| {
                                    this.torizon_setup.profile = index;
                                    cx.notify();
                                },
                            )),
                        )
                        .child(text(theme::CAPTION).text_color(t.muted).child(tr(note)));
                }
                body = body.child(text(theme::CAPTION).text_color(t.muted).child(tr("CPU workers: automatic. Rendering, simulation and background tasks use separate threads.")));
            }
            _ => {
                let on = self.settings.torizon_mascots.unwrap_or(true);
                body = body.child(text(theme::UI).child(tr("A little chaos, a lot of cats.")))
                    .child(div().flex().gap(px(12.)).items_center()
                        .child(switch("setup-cats", on, &t).on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                            this.settings.torizon_mascots = Some(!this.settings.torizon_mascots.unwrap_or(true)); cx.notify();
                        })))
                        .child(text(theme::UI).child(tr("Animated cat mascots"))))
                    .child(text(theme::CAPTION).text_color(t.muted).child(tr("Animations pause while the game is running. You can turn them off in Settings.")))
                    .child(text(theme::UI).child(tr("All set. Torizon is ready for your first world.")));
            }
        }
        let next = if step == 3 { "Start Torizon" } else { "Next" };
        let mut actions = div().flex().justify_between().items_center().gap(px(12.));
        if step > 0 {
            actions = actions.child(button("setup-back", "Back", Kind::Ghost, &t).on_click(
                cx.listener(|this, _: &ClickEvent, _, cx| {
                    this.torizon_setup.step = this.torizon_setup.step.saturating_sub(1);
                    cx.notify();
                }),
            ));
        } else {
            actions = actions.child(div().flex_1());
        }
        actions = actions.child(button("setup-next", next, Kind::Primary, &t).on_click(
            cx.listener(|this, _: &ClickEvent, _, cx| {
                this.setup_advance();
                cx.notify();
            }),
        ));
        let mut side = div()
            .w(px(230.))
            .flex_none()
            .flex()
            .flex_col()
            .gap(px(18.))
            .items_center()
            .child(text(theme::TITLE).child("Torizon"));
        if self.settings.torizon_mascots.unwrap_or(true) {
            side = side.child(self.torizon_mascot(184., step % 2 == 1));
        }
        let card = glass::plate(24.)
            .border_1()
            .border_color(t.rule)
            .w(px(860.))
            .p(px(28.))
            .flex()
            .gap(px(30.))
            .child(side)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(18.))
                    .child(text(theme::CAPTION).text_color(t.muted).child(trf(
                        "Step {step} of 4",
                        &[("step", &(step + 1).to_string())],
                    )))
                    .child(text(theme::HEADING).child(tr(TITLES[step])))
                    .child(body)
                    .children(
                        self.torizon_setup
                            .error
                            .clone()
                            .map(|error| text(theme::CAPTION).text_color(t.danger).child(error)),
                    )
                    .child(actions),
            );
        Some(
            div()
                .id("torizon-setup")
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .occlude()
                .child(card.relative().with_animation(
                    ElementId::named_usize("setup-enter", step),
                    motion::eased(motion::POP),
                    |d, p| d.opacity(p).top(px(motion::POP_RISE * (1. - p))),
                )),
        )
    }

    pub(super) fn torizon_options(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .mt(px(20.))
            .child(section_title("Torizon", "sliders", &t))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(
                        switch(
                            "torizon-cats",
                            self.settings.torizon_mascots.unwrap_or(true),
                            &t,
                        )
                        .on_click(cx.listener(
                            |this, _: &ClickEvent, _, cx| {
                                this.settings.torizon_mascots =
                                    Some(!this.settings.torizon_mascots.unwrap_or(true));
                                this.save_settings();
                                cx.notify();
                            },
                        )),
                    )
                    .child(text(theme::UI).child(tr("Animated cat mascots"))),
            )
            .child(
                button(
                    "torizon-setup-again",
                    "Run first-launch setup",
                    Kind::Secondary,
                    &t,
                )
                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                    this.settings.torizon_setup_done = false;
                    this.torizon_setup = Setup::default();
                    this.save_settings();
                    cx.notify();
                })),
            )
    }
}
