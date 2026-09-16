//! Main GUI component for the QR code generator.

use std::fmt::{self, Write};
use std::time::Duration;

use compio_log::warn;
use winio::prelude::*;

use crate::Result;
use crate::utils::{LOGICAL_CORES, pin_to_best_core, pin_to_core};
use crate::workload::sha256_workload;

/// Root component of the application UI.
pub struct MainModel {
    /// The main application window.
    window: Child<Window>,
    singlecore: Child<Button>,
    multicore: Child<Button>,
    pikafish: Child<Button>,
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

    /// Pikafish benchmark
    PikaStart,
    PikaComplete {
        nodes_per_sec: u32,
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

            pikafish: Button = (&window) => {
                text: "Pikafish",
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
            pikafish,
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
            },
            self.pikafish => {
                ButtonEvent::Click => MainMessage::PikaStart,
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

            MainMessage::PikaStart => {
                self.toggle_buttons(false)?;

                self.progress.set_pos(0)?;
                self.progress.set_maximum(49)?;
                compio::runtime::spawn_blocking({
                    let sender = sender.clone();
                    move || {
                        #[cfg(all(target_os = "android", target_arch = "aarch64"))]
                        {
                            use std::io::{BufRead, BufReader};
                            use std::path::PathBuf;
                            use std::process::Stdio;
                            use std::sync::Once;

                            use compio_log::error;

                            static RELEASED_FILES: Once = Once::new();

                            unsafe extern "C" {
                                fn getuid() -> u32;
                            }
                            let files = PathBuf::from(format!(
                                "/data/user/{}/io.github.mokurin000.minibench/files",
                                unsafe { getuid() / 100000 }
                            ));
                            let pikafish_exec = files.join("pikafish");
                            let pikafish_nnue = files.join("pikafish.nnue");

                            RELEASED_FILES.call_once(|| {
                                let pikafish =
                                    include_bytes!(concat!(env!("OUT_DIR"), "/pikafish"));
                                let pikafish_nnue_data =
                                    include_bytes!(concat!(env!("OUT_DIR"), "/pikafish.nnue"));

                                _ = std::fs::write(pikafish_exec, pikafish);
                                _ = std::fs::write(pikafish_nnue, pikafish_nnue_data);
                            });

                            let mut child = std::process::Command::new(pikafish_exec)
                                .arg("bench")
                                .current_dir(files)
                                .stdin(Stdio::null())
                                .stdout(Stdio::null())
                                .stderr(Stdio::piped())
                                .spawn()?;
                            let Some(stderr) = child.stderr.take() else {
                                error!("Failed to take stderr");
                                return;
                            };

                            let reader = BufReader::new(stderr);
                            for line in reader.lines() {
                                let Ok(line) = line else { break };
                                let line = line.trim();
                                match line {
                                    _ if line.starts_with("Position: ") => {
                                        sender.post(MainMessage::ProgressIncrease);
                                    }
                                    _ if line.starts_with("Nodes/second") => {
                                        if let Some(num) = line.split_whitespace().last() {
                                            if let Ok(nodes_per_sec) = num.parse() {
                                                sender.post(MainMessage::PikaComplete {
                                                    nodes_per_sec,
                                                });
                                            }
                                        } else {
                                            error!("Failed to capture measured NPS!");
                                            sender.post(MainMessage::PikaComplete {
                                                nodes_per_sec: 0,
                                            });
                                        }
                                    }
                                    _ => (),
                                }
                            }
                        }
                        #[cfg(not(all(target_os = "android", target_arch = "aarch64")))]
                        {
                            sender.post(MainMessage::PikaComplete { nodes_per_sec: 0 });
                        }
                    }
                })
                .detach();

                Ok(false)
            }
            MainMessage::PikaComplete { nodes_per_sec } => {
                self.toggle_buttons(true)?;
                if nodes_per_sec == 0 {
                    self.append_message(format_args!("[Pika] Unsupported"))?;
                } else {
                    self.append_message(format_args!("[Pika] {nodes_per_sec} Nodes/sec"))?;
                }
                Ok(true)
            }

            MainMessage::SingleComplete { kib_per_sec } => {
                self.toggle_buttons(true)?;

                let mb_per_sec = kib_per_sec * 1024.0 / 1_000_000.0;
                self.append_message(format_args!("SHA-256: {mb_per_sec:.01} MB/s"))?;

                Ok(true)
            }
            MainMessage::MultiComplete { kib_per_sec } => {
                self.toggle_buttons(true)?;

                let mb_per_sec = kib_per_sec * 1024.0 / 1_000_000.0;
                self.append_message(format_args!("[MT] SHA-256: {mb_per_sec:.01} MB/s"))?;

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
            self.pikafish => {
                margin: Margin::new_all_same(5.),
            },
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
        self.pikafish.set_enabled(enabled)?;

        Ok(())
    }
}
