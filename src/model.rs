//! Main GUI component for the QR code generator.

use std::fmt::{self, Write};
use std::time::Duration;

use compio_log::warn;
use winio::prelude::*;

use crate::Result;
use crate::utils::{LOGICAL_CORES, pin_to_best_core, pin_to_core, sha256_workload};

/// Root component of the application UI.
pub struct MainModel {
    /// The main application window.
    window: Child<Window>,
    singlecore: Child<Button>,
    multicore: Child<Button>,
    textbox: Child<TextBox>,
    progress: Child<Progress>,
}

pub enum MainMessage {
    /// Nothing to do
    Noop,
    /// Main window has been resized
    Resize,
    /// Theme changed
    ThemeChanged,
    /// Close main window
    Close,
    /// Start single-core test
    SingleStart,
    /// Complete single-core test
    SingleComplete {
        kib_per_sec: f64,
    },

    /// Multi-cores test
    MultiStart,
    /// Complete multi-cores test
    MultiComplete {
        kib_per_sec: f64,
    },

    StartTimer(Duration),
    ProgressIncrease,
}

impl Component for MainModel {
    type Error = color_eyre::Report;
    type Event = ();
    type Init<'a> = ();
    type Message = MainMessage;

    async fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        // Note: color-eyre does not enable VT100 on Windows on its own
        color_eyre::install()?;

        init! {
            window: Window = (()) => {
                text: "MiniBench",
                size: Size::new(300.0, 500.0),

                #[cfg(all(windows, feature = "winui"))]
                backdrop: Backdrop::Mica,
            },

            singlecore: Button = (&window) => {
                text: "SHA-256",
            },
            multicore: Button = (&window) => {
                text: "SHA-256 MT",
            },

            progress: Progress = (&window) => {
                minimum: 0,
                maximum: 100,
            },
            textbox: TextBox = (&window) => {
                readonly: true,
            },

        }

        window.show()?;

        Ok(Self {
            window,
            singlecore,
            multicore,
            textbox,
            progress,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        // listen to events
        start! {
            sender, default: MainMessage::Noop,
            self.window => {
                WindowEvent::Resize => MainMessage::Resize,
                WindowEvent::Close => MainMessage::Close,
                WindowEvent::ThemeChanged => MainMessage::ThemeChanged,
            },
            self.singlecore => {
                ButtonEvent::Click => MainMessage::SingleStart,
            },
            self.multicore => {
                ButtonEvent::Click => MainMessage::MultiStart,
            }
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        // update the window and functional children
        update_children!(self.window,)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        // deal with custom messages
        match message {
            MainMessage::Noop => Ok(false),
            MainMessage::ThemeChanged => Ok(false),
            MainMessage::Resize => Ok(true),
            MainMessage::Close => {
                // the root component output stops the application
                sender.output(());
                // need not to call `render`
                Ok(false)
            }
            MainMessage::SingleStart => {
                self.toggle_buttons(false)?;

                compio::runtime::spawn_blocking({
                    let sender = sender.clone();

                    move || {
                        if let Err(e) = pin_to_best_core() {
                            warn!("Failed to pin thread affinity: {e}");
                        }

                        let secs = 3;
                        let dur = Duration::from_secs(secs as u64);

                        sender.post(MainMessage::StartTimer(dur));

                        let (mibs, dur) = sha256_workload(dur);
                        let secs = dur.as_secs_f64();
                        let kib_per_sec = mibs as f64 / secs * 1024.0;

                        sender.post(MainMessage::SingleComplete { kib_per_sec });
                    }
                })
                .detach();

                Ok(false)
            }
            MainMessage::MultiStart => {
                self.toggle_buttons(false)?;

                compio::runtime::spawn_blocking({
                    let sender = sender.clone();

                    let secs = 3;
                    let dur = Duration::from_secs(secs as u64);

                    sender.post(MainMessage::StartTimer(dur));

                    move || {
                        let mut handles = vec![];
                        for lp in &*LOGICAL_CORES {
                            let os_id = lp.os_id;
                            handles.push(std::thread::spawn(move || {
                                if let Err(e) = pin_to_core(os_id) {
                                    warn!("Failed to pin thread to CPU {os_id}: {e}");
                                }
                                sha256_workload(dur)
                            }));
                        }

                        let kib_per_sec = handles
                            .into_iter()
                            .map(|handle| handle.join().expect("Thread error"))
                            .map(|(mibs, dur)| {
                                let secs = dur.as_secs_f64();
                                mibs as f64 / secs * 1024.0
                            })
                            .sum::<f64>();

                        sender.post(MainMessage::MultiComplete { kib_per_sec });
                    }
                })
                .detach();

                Ok(false)
            }

            MainMessage::SingleComplete { kib_per_sec } => {
                self.toggle_buttons(true)?;

                self.append_message(format_args!("SHA-256: {kib_per_sec:.01} KiB/s"))?;

                Ok(true)
            }
            MainMessage::MultiComplete { kib_per_sec } => {
                self.toggle_buttons(true)?;

                self.append_message(format_args!("SHA-256 MT: {kib_per_sec:.01} KiB/s"))?;

                Ok(true)
            }

            MainMessage::StartTimer(dur) => {
                self.progress.set_pos(0)?;
                self.start_timer(sender.clone(), dur);
                Ok(false)
            }
            MainMessage::ProgressIncrease => {
                let new = self.progress.pos()? + 1;
                self.progress.set_pos(new)?;
                Ok(false)
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let csize = self.window.client_size()?;

        let mut buttons = layout! {
            StackPanel::new(Orient::Horizontal),
            self.singlecore => {
                grow: true,
                margin: Margin::new_all_same(5.),
            },
            self.multicore => {
                grow: true,
                margin: Margin::new(5., 5., 5., 0.),
            },
        };
        let mut layout = layout! {
            StackPanel::new(Orient::Vertical),
            buttons,
            self.progress => {
                margin: Margin::new_all_same(5.),
            },
            self.textbox => {
                grow: true,
                margin: Margin::new_all_same(10.),
            },
        };

        layout.set_size(csize)?;
        Ok(())
    }

    fn render_children(&mut self) -> Result<()> {
        Ok(self.window.render()?)
    }
}

impl MainModel {
    fn append_message(&mut self, args: fmt::Arguments) -> Result<()> {
        let mut text = self.textbox.text()?;
        _ = text.write_fmt(args);
        _ = text.write_char('\n');
        self.textbox.set_text(text)?;

        Ok(())
    }

    fn start_timer(&self, sender: ComponentSender<Self>, duration: Duration) {
        let ms_per_interval = duration.as_millis() as u64 / 100;
        let dur = Duration::from_millis(ms_per_interval);

        compio::runtime::spawn_blocking(move || {
            for _ in 0..100 {
                std::thread::sleep(dur);
                sender.post(MainMessage::ProgressIncrease);
            }
        })
        .detach();
    }

    fn toggle_buttons(&mut self, enabled: bool) -> Result<()> {
        self.singlecore.set_enabled(enabled)?;
        self.multicore.set_enabled(enabled)?;

        Ok(())
    }
}
