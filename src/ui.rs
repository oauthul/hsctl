use std::fmt;
use eframe::{egui, egui::viewport::IconData};
use tracing::{warn, info, trace, debug};
use crate::{ThreadMessage, Response, sleep_sec, Commands};
use crossbeam_channel::{unbounded, Sender, Receiver};

fn make_new_rich_text(string: String, size: f32)

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
            Self::Unset => write!(formatting, "N/A"),
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


pub struct HeadsetGui {
    pub battery_level: GuiResponse,
    pub charging_status: GuiResponse,
    pub headset_status: GuiResponse,
    pub sidetone_status: GuiResponse,
    pub sidetone_volume: GuiResponse,
    pub noisegate_status: GuiResponse,
    pub microphone_status: GuiResponse,
    pub shutdown_time: GuiResponse,
    pub sender: Option<Sender<ThreadMessage>>,
    pub receiver: Option<Receiver<ThreadMessage>>,
    pub state: bool
}

impl HeadsetGui {
    fn new(cc: &eframe::CreationContext<'_>, gui_tx: Sender<ThreadMessage>, gui_rx: Receiver<ThreadMessage>) -> Self {
        Self {
            sender: Some(gui_tx),
            receiver: Some(gui_rx),
            ..Self::default()
        }
    }
}

impl Default for HeadsetGui {
    fn default() -> Self {
        Self {
            battery_level: GuiResponse::Unset,
            charging_status: GuiResponse::Unset,
            headset_status: GuiResponse::Unset,
            sidetone_status: GuiResponse::Unset,
            sidetone_volume: GuiResponse::Unset,
            noisegate_status: GuiResponse::Unset,
            microphone_status: GuiResponse::Unset,
            shutdown_time: GuiResponse::Unset,
            sender: None,
            receiver: None,
            state: false
        }
    }
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
            Ok(Box::new(HeadsetGui::new(cc, gui_tx, gui_rx)))
        }),
    )
}

impl eframe::App for HeadsetGui {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            if !self.state {
                ui.heading(egui::RichText::new("device is not synchronized")
                                        .size(24.0));

                if ui.add(egui::Button::new("Refresh")).clicked() {
                    trace!("refresh button clicked!");
                    self.sender
                        .clone()
                        .expect("failed to find sender variable for refresh!")
                        .send(ThreadMessage::Ready).unwrap();
                    trace!("sent gui ready command, awaiting device information");
                }
            } else {
                ui.heading(egui::RichText::new("device is synchronized")
                                        .size(24.0));
            }

            ui.separator();

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("device information:")
                                        .size(20.0));
                
                // device information
                ui.label(format!("battery level: {}", self.battery_level));
                ui.label(format!("charging status: {}", self.charging_status));
                ui.label(format!("headset status: {}", self.headset_status));
                ui.label(format!("auto-shutdown time: {}", self.shutdown_time));
                ui.label(format!("sidetone status: {}", self.sidetone_status));
                ui.label(format!("sidetone volume: {}", self.sidetone_volume));
                ui.label(format!("noisegate status: {}", self.noisegate_status));
                ui.label(format!("microphone status: {}", self.microphone_status));
            });


        });
    }

    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        match self.receiver
                .clone()
                .expect("failed to get required sender/receiver pair for gui!")
                .try_recv() {
            Ok(msg) => match msg {
                ThreadMessage::HeadsetResponse(resp) => match resp {
                    Response::BatteryLevel(level) => self.battery_level = GuiResponse::BatteryLevel(level),
                    Response::ChargingStatus(status) => self.charging_status = GuiResponse::ChargingStatus(status),
                    Response::IsActive(status) => self.headset_status = GuiResponse::IsActive(status),
                    Response::AutoShutdownTime(time) => self.shutdown_time = GuiResponse::AutoShutdownTime(time),
                    Response::SidetoneStatus(status) => self.sidetone_status = GuiResponse::SidetoneStatus(status),
                    Response::SidetoneVolume(volume) => self.sidetone_volume = GuiResponse::SidetoneVolume(volume),
                    Response::NoiseGateStatus(status) => self.noisegate_status = GuiResponse::NoiseGateStatus(status),
                    Response::MicrophoneStatus(status) => self.microphone_status = GuiResponse::MicrophoneStatus(status)
                },
                ThreadMessage::Request(_) => (),
                ThreadMessage::Set { command: _, param: _ } => (),
                ThreadMessage::Ready => {
                    info!("successfully synchronized gui and worker thread!");
                    self.state = true;
                }
            }
            Err(_) => ()
        }
    }
}