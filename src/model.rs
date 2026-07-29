//! Main GUI component for the QR code generator.

use winio::prelude::*;

use crate::Result;

/// Root component of the application UI.
pub struct MainModel {
    /// The main application window.
    window: Child<Window>,
    singlecore: Child<Button>,
    multicore: Child<Button>,
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
    /// Single-core test
    StartSingleCore,
    /// Multi-core test
    StartMultiCore,
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
        }

        window.show()?;

        Ok(Self {
            window,
            singlecore,
            multicore,
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
                ButtonEvent::Click => MainMessage::StartSingleCore,
            },
            self.multicore => {
                ButtonEvent::Click => MainMessage::StartMultiCore,
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
            MainMessage::StartSingleCore => Ok(false),
            MainMessage::StartMultiCore => Ok(false),
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
        };

        layout.set_size(csize)?;
        Ok(())
    }

    fn render_children(&mut self) -> Result<()> {
        Ok(self.window.render()?)
    }
}
