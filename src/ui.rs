use std::fmt;
use eframe::{egui, egui::viewport::IconData};
use tracing::{warn, info, trace, debug, instrument};
use crate::{ThreadMessage, Response, sleep_sec, Commands};
use crossbeam_channel::{unbounded, Sender, Receiver};
use winit;

fn new_rich_text(string: &str, size: f32) -> egui::RichText {
    egui::RichText::new(string).size(size)
}

pub enum GuiResponse {
    Unset,
    BatteryLevel(u8),
    ChargingStatus(bool),
    IsActive(bool),
    AutoShutdownTime(u8),
    SidetoneStatus(bool),
    SidetoneVolume(u8),
    NoiseGateStatus(bool),
    MicrophoneStatus(bool)
}

impl fmt::Display for GuiResponse {
    fn fmt(&self, formatting: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Unset => write!(formatting, "n/a"),
            Self::IsActive(activity) => write!(formatting, "{}", if *activity { "active" } else { "inactive" }),
            Self::BatteryLevel(level) => write!(formatting, "{}%", level),
            Self::ChargingStatus(status) => write!(formatting, "{}", if *status { "charging" } else { "not charging" }),
            Self::AutoShutdownTime(time) => write!(formatting, "{} minute(s)", time),
            Self::SidetoneStatus(status) => write!(formatting, "{}", if *status { "active" } else { "inactive" }),
            Self::SidetoneVolume(volume) => write!(formatting, "{}", volume),
            Self::NoiseGateStatus(status) => write!(formatting, "{}", if *status { "on" } else { "off" }),
            Self::MicrophoneStatus(status) => write!(formatting, "{}", if *status { "on" } else { "off" })
        }
    }
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

pub struct GuiDeviceInfo {
    pub battery_level: GuiResponse,
    pub charging_status: GuiResponse,
    pub headset_status: GuiResponse,
    pub sidetone_status: GuiResponse,
    pub sidetone_volume: GuiResponse,
    pub noisegate_status: GuiResponse,
    pub microphone_status: GuiResponse,
    pub shutdown_time: GuiResponse
}

impl Default for GuiDeviceInfo {
    fn default() -> Self {
        Self {
            battery_level: GuiResponse::Unset,
            charging_status: GuiResponse::Unset,
            headset_status: GuiResponse::Unset,
            sidetone_status: GuiResponse::Unset,
            sidetone_volume: GuiResponse::Unset,
            noisegate_status: GuiResponse::Unset,
            microphone_status: GuiResponse::Unset,
            shutdown_time: GuiResponse::Unset
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

pub struct Gui {
    channels: ThreadChannel,
    device: GuiDeviceInfo,
    state: State
}

pub fn main(gui_tx: Sender<ThreadMessage>, gui_rx: Receiver<ThreadMessage>) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "hsctl",
        options,
        Box::new(|cc| {
            Ok(Box::new(Gui::new(cc, gui_tx, gui_rx)))
        }),
    )
}

impl Gui {
    #[instrument(level = "debug", skip_all)]
    fn new(cc: &eframe::CreationContext<'_>, gui_tx: Sender<ThreadMessage>, gui_rx: Receiver<ThreadMessage>) -> Self {
        let mut context: eframe::egui::Context;
        context = cc.egui_ctx.clone();
        
        gui_tx.clone().send(ThreadMessage::Context(context)).unwrap();
        trace!("sent context handle to worker thread");

        gui_tx.clone().send(ThreadMessage::Ready).unwrap();
        trace!("sent ready state to worker thread");

        Self {
            channels: ThreadChannel {
                sender: Some(gui_tx),
                receiver: Some(gui_rx)
            },
            device: GuiDeviceInfo::default(),
            state: State::default()
        }
    }
}

impl eframe::App for Gui {
    #[instrument(level = "debug", skip_all)]
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            if !self.state.is_synced {
                ui.heading(new_rich_text("device is not synchronized.", 24.0));
            } else {
                ui.heading(new_rich_text("device is synchronized!", 24.0));
            }

            ui.separator();

            ui.vertical(|ui| {
                ui.label(new_rich_text("device information:", 20.0));
                
                // device information
                ui.label(format!("battery level: {}", self.device.battery_level));
                ui.label(format!("charging status: {}", self.device.charging_status));
                ui.label(format!("headset status: {}", self.device.headset_status));
                ui.label(format!("auto-shutdown time: {}", self.device.shutdown_time));
                ui.label(format!("sidetone status: {}", self.device.sidetone_status));
                ui.label(format!("sidetone volume: {}", self.device.sidetone_volume));
                ui.label(format!("noisegate status: {}", self.device.noisegate_status));
                ui.label(format!("microphone status: {}", self.device.microphone_status));
            });
        });
    }

    #[instrument(level = "debug", skip_all)]
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        match self.channels.receiver
                .clone()
                .expect("failed to get required sender/receiver pair for gui!")
                .try_recv() {
            Ok(msg) => match msg {
                ThreadMessage::HeadsetResponse(resp) => match resp {
                    Response::BatteryLevel(level) => self.device.battery_level = GuiResponse::BatteryLevel(level),
                    Response::ChargingStatus(status) => self.device.charging_status = GuiResponse::ChargingStatus(status),
                    Response::IsActive(status) => self.device.headset_status = GuiResponse::IsActive(status),
                    Response::AutoShutdownTime(time) => self.device.shutdown_time = GuiResponse::AutoShutdownTime(time),
                    Response::SidetoneStatus(status) => self.device.sidetone_status = GuiResponse::SidetoneStatus(status),
                    Response::SidetoneVolume(volume) => self.device.sidetone_volume = GuiResponse::SidetoneVolume(volume),
                    Response::NoiseGateStatus(status) => self.device.noisegate_status = GuiResponse::NoiseGateStatus(status),
                    Response::MicrophoneStatus(status) => self.device.microphone_status = GuiResponse::MicrophoneStatus(status)
                },
                ThreadMessage::Request(_) => (),
                ThreadMessage::Set { command: _, param: _ } => (),
                ThreadMessage::Ready => {
                    info!("successfully synchronized gui and worker thread!");
                    self.state.is_synced = true;
                },
                _ => ()
            }
            Err(_) => ()
        }
    }
}