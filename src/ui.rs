use std::fmt;
use eframe::{egui, UserEvent, egui::Context};
use tracing::{warn, trace, instrument};
use crate::{ThreadMessage, hardware::Response, Arc, Mutex};
use crossbeam_channel::{Sender, Receiver};
use winit::event_loop::{ControlFlow, EventLoop};

fn new_rich_text(string: &str, size: f32) -> egui::RichText {
    egui::RichText::new(string).size(size)
}

fn send_context(context: Context, sender: Sender<ThreadMessage>) -> Result<(), ()> {
    sender.clone().send(ThreadMessage::Context(context)).unwrap();

    Ok(())
}

fn get_info_and_sync(
    device: Arc<Mutex<GuiDeviceInfo>>,
    sender: Sender<ThreadMessage>,
    receiver: Receiver<ThreadMessage>,
    cached_state: &mut GuiDeviceInfo,
    state: &mut State) -> Result<(), ()>    
{
    sender.send(ThreadMessage::GuiDeviceStruct(device.clone())).unwrap();
    sender.clone().send(ThreadMessage::Ready).unwrap();
    trace!("sent ready state to worker thread");

    loop {
        match receiver.recv() {
            Ok(resp) => match resp {
                ThreadMessage::Ready => {
                    trace!("worker thread and ui thread are in sync!");
                    state.is_synced = true;
                    break
                },
                _ => ()
            },

            Err(_) => ()
        }
    }

    trace!("broke from loop! cached information: {:?}", device);
    if let Ok(snapshot) = device.try_lock() {
            *cached_state = snapshot.clone();
    }

    Ok(())
}

pub struct ThreadChannel {
    pub sender: Option<Sender<ThreadMessage>>,
    pub receiver: Option<Receiver<ThreadMessage>>
}

impl Default for ThreadChannel {
    fn default() -> Self {
        Self {
            sender: None,
            receiver: None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiDeviceInfo {
    pub battery_level: Response,
    pub charging_status: Response,
    pub headset_status: Response,
    pub sidetone_status: Response,
    pub sidetone_volume: Response,
    pub noisegate_status: Response,
    pub microphone_status: Response,
    pub shutdown_time: Response
}

impl Default for GuiDeviceInfo {
    fn default() -> Self {
        Self {
            battery_level: Response::Unset,
            charging_status: Response::Unset,
            headset_status: Response::Unset,
            sidetone_status: Response::Unset,
            sidetone_volume: Response::Unset,
            noisegate_status: Response::Unset,
            microphone_status: Response::Unset,
            shutdown_time: Response::Unset
        }
    }
}

pub struct State {
    is_synced: bool
}

impl Default for State {
    fn default() -> Self {
        Self {
            is_synced: false
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", if self.is_synced { "synchronized" } else { "unsynchronized" })
    }
}

pub struct Gui {
    channels: ThreadChannel,
    device: Arc<Mutex<GuiDeviceInfo>>,
    state: State,
    cached: GuiDeviceInfo
}

pub fn main(gui_tx: Sender<ThreadMessage>, gui_rx: Receiver<ThreadMessage>) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 400.0]),
        ..Default::default()
    };

    let eventloop = EventLoop::<UserEvent>::with_user_event().build()?;
    eventloop.set_control_flow(ControlFlow::Poll);

    let mut winit_app = eframe::create_native(
        "hsctl",
        options,
        Box::new(|cc| {
            Ok(Box::new(Gui::new(cc, gui_tx, gui_rx)))
        }),
        &eventloop
    );

    eventloop.run_app(&mut winit_app)?;

    Ok(())
}

impl Gui {
    #[instrument(level = "debug", skip_all)]
    fn new(cc: &eframe::CreationContext<'_>, gui_tx: Sender<ThreadMessage>, gui_rx: Receiver<ThreadMessage>) -> Self {
        let mut device: Arc<Mutex<GuiDeviceInfo>> = Arc::new(Mutex::new(GuiDeviceInfo::default()));
        let mut state: State = State::default();
        let mut cached_state: GuiDeviceInfo = GuiDeviceInfo::default();

        let context: eframe::egui::Context = cc.egui_ctx.clone();
        if let Ok(_) = send_context(context, gui_tx.clone()) {
            trace!("sent context handle to worker thread")
        }

        if let Ok(_) = get_info_and_sync(device.clone(), gui_tx.clone(), gui_rx.clone(), &mut cached_state, &mut state) {
            trace!("got device information and synchronized ui and worker threads")
        }

        Self {        
            channels: ThreadChannel {
                sender: Some(gui_tx),
                receiver: Some(gui_rx)
            },
            device: device,
            state: state,
            cached: cached_state
        }
    }
}

impl eframe::App for Gui {
    #[instrument(level = "debug", skip_all)]
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Ok(snapshot) = self.device.try_lock() {
            self.cached = snapshot.clone();
        }
        
        let devstate = format!("current device state: {}", self.state);

        egui::CentralPanel::default().show(ui, |ui| {
            ui.label(new_rich_text(&devstate, 24.0));

            if self.cached == GuiDeviceInfo::default() {
                ui.label(new_rich_text("device information not in sync yet", 24.0));
            } else {
                ui.vertical(|ui| {
                    ui.label(new_rich_text("device information:", 24.0));
                    ui.label(format!("{}", self.cached.battery_level));
                    ui.label(format!("{}", self.cached.charging_status));
                    ui.label(format!("{}", self.cached.headset_status));
                    ui.label(format!("{}", self.cached.shutdown_time));
                    ui.label(format!("{}", self.cached.sidetone_status));
                    ui.label(format!("{}", self.cached.sidetone_volume));
                    ui.label(format!("{}", self.cached.noisegate_status));
                    ui.label(format!("{}", self.cached.microphone_status));
                });
            }           
        });
    }

}