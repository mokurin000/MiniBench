//! Main GUI component for the QR code generator.

use std::fmt::{self, Write};
use std::time::Duration;

use compio_log::warn;
use winio::prelude::*;

use crate::Result;
use crate::utils::{pin_to_best_core, sha256_workload};

/// Root component of the application UI.
pub struct MainModel {
    /// The main application window.
    window: Child<Window>,
    singlecore: Child<Button>,
    multicore: Child<Button>,
    textbox: Child<TextBox>,
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
    SingleComplete { mibs: usize, secs: usize },
    /// Multi-cores test
    MultiStart,
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
                text: "Single-Thread",
            },
            multicore: Button = (&window) => {
                text: "Multi-Thread",
            },
            textbox: TextBox = (&window) => {
                text: "Waiting for further command...\n",
                readonly: true,
            }
        }

        window.show()?;

        Ok(Self {
            window,
            singlecore,
            multicore,
            textbox,
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
                self.singlecore.disable()?;
                self.multicore.disable()?;

                compio::runtime::spawn_blocking({
                    let sender = sender.clone();

                    move || {
                        if let Err(e) = pin_to_best_core() {
                            warn!("Failed to pin thread affinity: {e}");
                        }

                        let secs = 10;
                        let mibs = sha256_workload(Duration::from_secs(secs as u64));

                        sender.post(MainMessage::SingleComplete { mibs, secs });
                    }
                })
                .detach();

                Ok(false)
            }
            MainMessage::SingleComplete { mibs, secs } => {
                self.singlecore.enable()?;
                self.multicore.enable()?;

                let speed = mibs as f64 / secs as f64;
                self.append_message(format_args!("Single-Thread: {speed:.02} MiB/s"))?;

                Ok(true)
            }
            MainMessage::MultiStart => {
                self.append_message(format_args!("Multi-Thread unsupported yet."))?;
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
}
