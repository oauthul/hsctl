use std::fmt;
use eframe::{egui, egui::viewport::IconData};
use tracing::{warn, info, trace, debug};
use crate::{ThreadMessage, Response, sleep_sec, Commands};
use crossbeam_channel::{unbounded, Sender, Receiver};

pub struct HeadsetGuiValues {
    pub battery_level: Option<u8>,
    pub charging_status: Option<bool>,
    pub headset_status: Option<bool>,
    pub sidetone_status: Option<bool>,
    pub sidetone_volume: Option<u8>,
    pub noisegate_status: Option<bool>,
    pub microphone_status: Option<bool>,
    pub shutdown_time: Option<u8>,
    pub sender: Option<Sender<ThreadMessage>>,
    pub receiver: Option<Receiver<ThreadMessage>>,
    pub state: bool
}

impl HeadsetGuiValues {
    fn new(cc: &eframe::CreationContext<'_>, gui_tx: Sender<ThreadMessage>, gui_rx: Receiver<ThreadMessage>) -> Self {
        gui_tx.send(ThreadMessage::Ready).unwrap();

        Self {
            sender: Some(gui_tx),
            receiver: Some(gui_rx),
            ..Self::default()
        }
    }
}

impl Default for HeadsetGuiValues {
    fn default() -> Self {
        Self {
            battery_level: None,
            charging_status: None,
            headset_status: None,
            sidetone_status: None,
            sidetone_volume: None,
            noisegate_status: None,
            microphone_status: None,
            shutdown_time: None,
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
        "hyperx-monitor",
        options,
        Box::new(|cc| {
            Ok(Box::new(HeadsetGuiValues::new(cc, gui_tx, gui_rx)))
        }),
    )
}

impl eframe::App for HeadsetGuiValues {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            if self.state == false {
                ui.heading(egui::RichText::new("device is not synchronized.")
                                        .size(24.0));
            }

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("device information:")
                                        .size(20.0));
                
                // device information
                ui.label(format!("battery level: {}", self.battery_level
                                                                        .map(|level| level.to_string())
                                                                        .unwrap_or_else(|| "loading".to_string())));

                ui.label(format!("charging status: {}", self.charging_status
                                                                            .map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));

                ui.label(format!("headset status: {}", self.headset_status
                                                                            .map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));

                ui.label(format!("auto-shutdown time: {}", self.shutdown_time
                                                                            .map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));

                ui.label(format!("sidetone status: {}", self.sidetone_status
                                                                            .map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));

                ui.label(format!("sidetone volume: {}", self.sidetone_volume
                                                                            .map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));

                ui.label(format!("noisegate status: {}", self.noisegate_status
                                                                            .map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));


                ui.label(format!("microphone status: {}", self.microphone_status
                                                                            .map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));
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
                    Response::BatteryLevel(level) => self.battery_level = Some(level),
                    Response::ChargingStatus(status) => self.charging_status = Some(status),
                    Response::IsActive(status) => self.headset_status = Some(status),
                    Response::AutoShutdownTime(time) => self.shutdown_time = Some(time),
                    Response::SidetoneStatus(status) => self.sidetone_status = Some(status),
                    Response::SidetoneVolume(volume) => self.sidetone_volume = Some(volume),
                    Response::NoiseGateStatus(status) => self.noisegate_status = Some(status),
                    Response::MicrophoneStatus(status) => self.microphone_status = Some(status)
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