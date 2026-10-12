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
    pub receiver: Option<Receiver<ThreadMessage>>
}

pub struct State {
    pub ready: bool
}

impl HeadsetGuiValues {
    pub fn default() -> Self {
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
            receiver: None
        }
    }

    pub fn with_sender(send_var: Sender<ThreadMessage>) -> Self {
        Self {
            sender: Option<send_var>,
            ..Self::default()
        }
    }

    pub fn with_receiver(recv_var: Receiver<ThreadMessage>) -> Self {
        Self {
            receiver: Option<recv_var>,
            ..Self::default()

        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            ready: false
        }
    }
}

pub fn main(gui_tx: Sender<ThreadMessage>, gui_rx: Receiver<ThreadMessage>) -> eframe::Result {
    trace!("trying to initialize gui");

    let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                                            .with_inner_size([900.0, 400.0])
                                            .with_icon(IconData::default()),
            ..Default::default()
    };

    let mut values = HeadsetGuiValues::default();
    let mut state = State::default();

    if let Ok(att) = gui_tx.send(ThreadMessage::Ready) {
        debug!("gui is ready, attempting to get information from headset");
    };

    eframe::run_ui_native("hyperx-monitor", options, move |ui, _frame| {
        egui::CentralPanel::default().show(ui, |ui| {
            let (gui_tx_clone, gui_rx_clone) = (gui_tx.clone(), gui_rx.clone());

            match gui_rx_clone.try_recv() {
                Ok(msg) => match msg {
                    ThreadMessage::HeadsetResponse(resp) => match resp {
                        Response::BatteryLevel(level) => values.battery_level = Some(level),
                        Response::ChargingStatus(status) => values.charging_status = Some(status),
                        Response::IsActive(status) => values.headset_status = Some(status),
                        Response::AutoShutdownTime(time) => values.shutdown_time = Some(time),
                        Response::SidetoneStatus(status) => values.sidetone_status = Some(status),
                        Response::SidetoneVolume(volume) => values.sidetone_volume = Some(volume),
                        Response::NoiseGateStatus(status) => values.noisegate_status = Some(status),
                        Response::MicrophoneStatus(status) => values.microphone_status = Some(status)
                    },
                    ThreadMessage::Request(_) => (),
                    ThreadMessage::Set { command: _, param: _ } => (),
                    ThreadMessage::Ready => {
                        info!("successfully synchronized gui and worker thread!");
                        state.ready = true;
                    }
                }
                Err(_) => ()
            }

            if state.ready == false {
                ui.label(egui::RichText::new("device is currently not connected!\n").size(20.0));
            };

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("device information:")
                                        .size(20.0));
                
                // device information
                ui.label(format!("battery level: {}", values.battery_level
                                                                        .map(|level| level.to_string())
                                                                        .unwrap_or_else(|| "loading".to_string())));

                ui.label(format!("charging status: {}", values.charging_status
                                                                            .map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));

                ui.label(format!("headset status: {}", values.headset_status
                                                                            .map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));

                ui.label(format!("auto-shutdown time: {}", values.shutdown_time
                                                                            .map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));

                ui.label(format!("sidetone status: {}", values.sidetone_status
                                                                            .map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));

                ui.label(format!("sidetone volume: {}", values.sidetone_volume
                                                                            .map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));

                ui.label(format!("noisegate status: {}", values.noisegate_status
                                                                            .map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));


                ui.label(format!("microphone status: {}", values.microphone_status
                                                                            .map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));
            });
        });
    })
}