use std::fmt;
use eframe::egui::Context;
use tracing::{info, debug, warn, error, trace, instrument};
use hidapi::{HidApi, HidDevice, HidError, DeviceInfo};
use std::collections::HashMap;
use clap::{Subcommand};
use std::thread::sleep;
use std::time::Duration;
use crossbeam_channel::{unbounded, Sender, Receiver};
use crate::{ThreadMessage, sleep_ms, sleep_sec,ThreadMessage::HeadsetResponse};

pub const USAGE_PAGE: u16 = 0xFF90;
pub const VENDOR_ID: u16 = 0x03F0;
pub const PRODUCT_ID: u16 = 0x0D93;
pub const BATTERY_LEVEL_POS: usize = 7;
pub const DEFAULT_RESP_POS: usize = 4;
pub const COMMAND_POS: usize = 3;
pub const READ_SUCCESS: u8 = 20;
pub const READ_EMPTY: usize = 0;
pub const READ_EMPTY_U8: u8 = 0;

macro_rules! execute {
    ($self:ident, $field:ident, $command:expr) => {
        let $field = $self.query($command, None)?;
        info!("{}", $field);
        $self.$field = Some($field)
    };

    ($self:ident, $field:ident, $command:expr, $value:expr) => {
        let $field = $self.query($command, $value)?;
        info!("{}", $field);
        $self.$field = Some($field)
    }
}

macro_rules! send_to_gui {
    ($self:ident, $field:ident, $sender:ident) => {
        if let Some(resp) = $self.$field 
        {
            match $sender.send(ThreadMessage::HeadsetResponse(resp)) {
                Ok(_) => trace!("sending '{:?}' to gui thread", resp),
                Err(error) => {
                    warn!("failed to send data to gui! {}", error);
                    return Err(HeadsetError::SendToGuiFail { attempted_data: resp })
                }
            }
        }
    }
}

#[repr(u8)]
#[derive(Debug, Subcommand)]
pub enum Commands {
    // To set the Noise Gate status use the SetSidetoneStatus command,
    // though it may not do anything as per localized testing and may only modify the Sidetone.

    GetHeadsetStatus = 1,
    GetBatteryLevel = 2,
    GetChargingStatus = 3,
    GetMicrophoneStatus = 5,
    GetSidetoneStatus = 6,
    GetAutoShutdownTime = 7,
    GetSidetoneVolume = 11,
    GetNoiseGateStatus = 13,
    SetSidetoneStatus = 33,
    SetAutoShutdownTime = 34,
    SetSidetoneVolume = 35
}

impl TryFrom<u8> for Commands {
    type Error = String;
    #[instrument(level = "debug", skip_all)]
    fn try_from(command: u8) -> Result<Self, Self::Error> {
        match command {
            1 => Ok(Self::GetHeadsetStatus),
            2 => Ok(Self::GetBatteryLevel),
            3 => Ok(Self::GetChargingStatus),
            5 => Ok(Self::GetMicrophoneStatus),
            6 => Ok(Self::GetSidetoneStatus),
            7 => Ok(Self::GetAutoShutdownTime),
            11 => Ok(Self::GetSidetoneVolume),
            13 => Ok(Self::GetNoiseGateStatus),
            32 => Ok(Self::GetMicrophoneStatus),
            33 => Ok(Self::SetSidetoneStatus),
            34 => Ok(Self::SetAutoShutdownTime),
            35 => Ok(Self::SetSidetoneVolume),
            _ => Err(format!("no command match for response {} found!", command))
        }

    }
}

impl Commands {
    #[instrument(skip_all)]
    pub fn parse(buf: &[u8; 20]) -> Result<Response, HeadsetError> {
        trace!("parsing buffer: {:?}", buf);

        let command: Commands;
        let command_buf = buf[COMMAND_POS];
        let mut resp: u8 = 0;

        match buf[BATTERY_LEVEL_POS] {
            READ_EMPTY_U8 => resp = buf[DEFAULT_RESP_POS],
            1..=100 => resp = buf[BATTERY_LEVEL_POS],
            _ => ()
        }

        match Self::try_from(command_buf) {
            Ok(command_name) => {
                command = command_name;
                trace!("command found: {:?}", command);
            },
            Err(error) => {
                return Err(HeadsetError::ParseError { msg: error })
            }
        }

        match command {
            Self::GetChargingStatus => match resp {
                1 => Ok(Response::ChargingStatus(true)),
                0 => Ok(Response::ChargingStatus(false)),
                _ => Err(HeadsetError::ParseError { msg: format!("invalid response value: {}", resp) })
            },

            Self::GetHeadsetStatus => match resp {
                1 => Ok(Response::IsActive(true)),
                4 => Ok(Response::IsActive(true)),
                3 => Ok(Response::IsActive(false)),
                _ => Err(HeadsetError::ParseError { msg: format!("invalid response value: {}", resp) })
            },

            Self::GetBatteryLevel => match resp {
                0..=100 => Ok(Response::BatteryLevel(resp)),
                _ => Err(HeadsetError::ParseError { msg: format!("invalid response value: {}", resp) })
            },

            Self::GetMicrophoneStatus => match resp {
                1 => Ok(Response::MicrophoneStatus(false)),
                0 => Ok(Response::MicrophoneStatus(true)),
                _ => Err(HeadsetError::ParseError { msg: format!("invalid response value: {}", resp) })
            },

            Self::GetNoiseGateStatus => match resp {
                1 => Ok(Response::NoiseGateStatus(false)),
                0 => Ok(Response::NoiseGateStatus(true)),
                _ => Err(HeadsetError::ParseError { msg: format!("invalid response value: {}", resp) })
            },

            Self::GetSidetoneStatus => match resp {
                1 => Ok(Response::SidetoneStatus(true)),
                0 => Ok(Response::SidetoneStatus(false)),
                _ => Err(HeadsetError::ParseError { msg: format!("invalid response value: {}", resp) })
            },

            Self::SetSidetoneStatus => match resp {
                1 => Ok(Response::SidetoneStatus(true)),
                0 => Ok(Response::SidetoneStatus(false)),
                _ => Err(HeadsetError::ParseError { msg: format!("invalid response value: {}", resp) })
            },

            Self::GetSidetoneVolume => match resp {
                0..=u8::MAX => Ok(Response::SidetoneVolume(resp)),
            },

            Self::SetSidetoneVolume => match resp {
                0..=u8::MAX => Ok(Response::SidetoneVolume(resp)),
            },

            Self::GetAutoShutdownTime => match resp {
                0..=u8::MAX => Ok(Response::AutoShutdownTime(resp)),
            },

            Self::SetAutoShutdownTime => match resp {
                0..=u8::MAX => Ok(Response::AutoShutdownTime(resp)),
            },

        }

    }
}

#[derive(Debug, PartialEq)]
pub enum HeadsetError {
    Disconnected,
    SendToGuiFail { attempted_data: Response },
    WriteError,
    ReadError { read: u8 },
    HidApiError { msg: String },
    EmptyHandle,
    FlushError,
    ParseError { msg: String },
    GetContextError
}

impl fmt::Display for HeadsetError {
    fn fmt(&self, formatting: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SendToGuiFail { attempted_data } => write!(formatting, "failed to send data to gui thread!"),
            Self::Disconnected => write!(formatting, "device disconnected!"),
            Self::WriteError => write!(formatting, "failed to write to device!"),
            Self::ReadError { read } => write!(formatting, "failed to read data! expected: {}, got: {}", READ_SUCCESS, read),
            Self::HidApiError { msg } => write!(formatting, "hidapi error occurred, msg: {}", msg),
            Self::EmptyHandle => write!(formatting, "no matching usb device found!"),
            Self::FlushError => write!(formatting, "failed to flush queue!"),
            Self::ParseError { msg } => write!(formatting, "failed to parse buffer! msg: {}", msg),
            Self::GetContextError => write!(formatting, "failed to get context handle from gui!")
        }
    }
}

#[derive(Debug)]
pub struct HeadsetInfo {
    pub handle: HidDevice,
    pub battery_level: Option<Response>,
    pub charging_status: Option<Response>,
    pub headset_status: Option<Response>,
    pub sidetone_status: Option<Response>,
    pub sidetone_volume: Option<Response>,
    pub noisegate_status: Option<Response>,
    pub microphone_status: Option<Response>,
    pub shutdown_time: Option<Response>
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Response {
    BatteryLevel(u8),
    ChargingStatus(bool),
    IsActive(bool),
    AutoShutdownTime(u8),
    SidetoneStatus(bool),
    SidetoneVolume(u8),
    NoiseGateStatus(bool),
    MicrophoneStatus(bool)
}


impl fmt::Display for Response {
    fn fmt(&self, formatting: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IsActive(activity) => write!(formatting, "device status: {}", if *activity { "active" } else { "inactive" }),
            Self::BatteryLevel(level) => write!(formatting, "battery level: {}%", level),
            Self::ChargingStatus(status) => write!(formatting, "charging status: {}", if *status { "charging" } else { "not charging" }),
            Self::AutoShutdownTime(time) => write!(formatting, "auto-shutdown delay: {} minute(s)", time),
            Self::SidetoneStatus(status) => write!(formatting, "sidetone status: {}", if *status { "active" } else { "inactive" }),
            Self::SidetoneVolume(volume) => write!(formatting, "sidetone volume: {}", volume),
            Self::NoiseGateStatus(status) => write!(formatting, "noisegate status: {}", if *status { "on" } else { "off" }),
            Self::MicrophoneStatus(status) => write!(formatting, "microphone status: {}", if *status { "on" } else { "off" })
        }
    }
}

impl HeadsetInfo {
    #[instrument(skip_all)]
    pub fn new() -> Result<Self, HeadsetError> {
        let mut api = hidapi::HidApi::new()
            .map_err(|error| HeadsetError::HidApiError { msg: error.to_string() })?;

        Ok(Self {
            handle: Self::get_handle(&mut api)?,
            battery_level: None,
            charging_status: None,
            headset_status: None,
            sidetone_status: None,
            sidetone_volume: None,
            noisegate_status: None,
            microphone_status: None,
            shutdown_time: None
        })
    }

    #[instrument(level = "debug", skip_all)]
    pub fn query(&mut self, command: Commands, param: Option<u8>) -> Result<Response, HeadsetError> {
        let mut buf = [0u8; 20];
        let mut timeout: u8 = 0;
        self.flush_queue(buf)?;
        self.execute_command(command, param)?;
        sleep_ms(100);

        loop {
            let read = self.handle.read_timeout(&mut buf, 1000);
            match read {
                Ok(READ_EMPTY) => {
                    trace!("reading no data from device. attempt: #{}", timeout);
                    timeout += 1;
                    if timeout == 2 {
                        trace!("reading {} bytes, device may be disconnected", READ_EMPTY);
                        return Err(HeadsetError::Disconnected)
                    }
                },
                Ok(bytes) => {
                    debug!("read {} bytes", bytes);
                    break
                },
                Err(error) => return Err(HeadsetError::HidApiError { msg: error.to_string() })
            }
        }
        

        trace!(response = ?buf);
        let resp = Commands::parse(&buf)?;
        sleep_ms(100);
        self.flush_queue(buf)?;
        
        Ok(resp)
    }

    #[instrument(level = "trace", skip_all)]
    pub fn flush_queue(&self, mut buf: [u8; 20]) -> Result<(), HeadsetError> {
        trace!("attempting to set device to non-blocking mode");

        match self.handle.set_blocking_mode(false) {
            Ok(_) => debug!("set the device to non-blocking mode"),
            Err(error) => {
                warn!("error occurred while setting blocking mode: {}", error);
                return Err(HeadsetError::HidApiError { msg: error.to_string() })
            }
        }

        loop {
            match self.handle.read(&mut buf) {
                Ok(READ_EMPTY) => 
                {   
                    trace!(buffer = ?buf);
                    debug!("successfully flushed buffer queue");
                    trace!("attempting to set device to blocking mode");
                    match self.handle.set_blocking_mode(true) {
                        Ok(_) => {
                            debug!("reset the device to blocking mode");
                            trace!("stopping buffer queue flush");
                            break
                        },
                        Err(error) => { 
                            warn!("error occurred while setting blocking mode: {}", error);
                            return Err(HeadsetError::HidApiError { msg: error.to_string() })
                        }   
                        
                    }
                },
                Ok(bytes) => { 
                    trace!(buffer = ?buf, bytes_read = bytes);
                },
                Err(error) => return Err(HeadsetError::HidApiError { msg: error.to_string() })
            }
        }

        Ok(())
    }


    #[instrument(level = "debug", skip_all)]
    pub fn execute_command(&self, command: Commands, param: Option<u8>) -> Result<(), HeadsetError> {
        trace!(command = ?command, param = param, "called execute method");

        let mut buf = [0u8; 20];
        buf[0] = 0x06; // Report ID
        buf[1] = 0xFF; // Constant
        buf[2] = 0xBB; // Constant
        buf[COMMAND_POS] = command as u8;

        if let Some(value) = param {
            buf[DEFAULT_RESP_POS] = value;
        }
        
        trace!(buffer = ?buf, "sending output report to device");
        match self.handle.send_output_report(&buf) {
            Ok(()) => {
                debug!("successfully wrote command to device");
                sleep_ms(50);
                Ok(())
            }
            Err(error) => {
                Err(HeadsetError::HidApiError { msg: error.to_string() })
            }
        }

    }

    #[instrument(level = "trace", skip_all)]
    pub fn query_device(&mut self) -> Result<(), HeadsetError> {
        execute!(self, battery_level, Commands::GetBatteryLevel);
        execute!(self, headset_status, Commands::GetHeadsetStatus);
        execute!(self, charging_status, Commands::GetChargingStatus);
        execute!(self, microphone_status, Commands::GetMicrophoneStatus);
        execute!(self, shutdown_time, Commands::GetAutoShutdownTime);
        execute!(self, sidetone_status, Commands::GetSidetoneStatus);
        execute!(self, sidetone_volume, Commands::GetSidetoneVolume);
        execute!(self, noisegate_status, Commands::GetNoiseGateStatus);
        
        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_handle(api: &mut HidApi) -> Result<HidDevice, HeadsetError> {
        match api.refresh_devices() {
            Ok(_) => trace!("refreshed usb devices"),
            Err(error) => return Err(HeadsetError::HidApiError { msg: error.to_string() })
        }

        let target = api.device_list().find(|&target| {
            target.product_id() == PRODUCT_ID &&
            target.vendor_id() == VENDOR_ID &&
            target.usage_page() == USAGE_PAGE
        });

        match target {
            None => Err(HeadsetError::EmptyHandle),
            Some(target) => match target.open_device(&api) {
                Ok(device) => {
                    debug!("got device handle successfully");
                    Ok(device)
                },
                Err(error) => Err(HeadsetError::HidApiError { msg: error.to_string() }),
            }
        }
    }
    
    #[instrument(level = "debug", skip_all)]
    pub fn about_device(&self) -> Result<(), HeadsetError> {
        debug!("grabbing device information");

        match self.handle.get_device_info() {
            Ok(info) => {
                info!("{:-^60}", " device information from hidapi ");
                info!("{:<27} {}", "product name:", info.product_string().unwrap_or("unknown"));
                info!("{:<27} {:#X}", "usage page:", info.usage_page());
                info!("{:<27} {:#X}", "usage:", info.usage());
                info!("{:<27} {:#X}", "vendor id:", info.vendor_id());
                info!("{:<27} {:#X}", "product id:", info.product_id());
                info!("{:<27} {:?}", "connection:", info.bus_type());
                info!("{:<27} {}", "connected interface:", info.interface_number());
                info!("{:-^60}", " end of info ");
                Ok(())
            }
            Err(error) => Err(HeadsetError::HidApiError { msg: error.to_string() })
        }
    }

    #[instrument(level = "debug", skip_all)]
    pub fn send_info_to_gui(&mut self, sender: Sender<ThreadMessage>) -> Result<(), HeadsetError> {
        // self.simulate_connect();
        
        send_to_gui!(self, battery_level, sender);
        send_to_gui!(self, charging_status, sender);
        send_to_gui!(self, headset_status, sender);
        send_to_gui!(self, sidetone_status, sender);
        send_to_gui!(self, sidetone_volume, sender);
        send_to_gui!(self, noisegate_status, sender);
        send_to_gui!(self, microphone_status, sender);
        send_to_gui!(self, shutdown_time, sender);
        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_context_handle(&mut self, hardware_rx: Receiver<ThreadMessage>) -> Result<Context, HeadsetError> {
        loop {
            match hardware_rx.recv() {
                Ok(resp) => {
                    trace!("waiting until gui context handle is received..");
                    match resp {
                        ThreadMessage::Context(ctx) => {
                            debug!("got gui context handle");
                            return Ok(ctx)
                        },
                        _ => ()
                    }
                },

                Err(_) => return Err(HeadsetError::GetContextError)
            }
        }
    }

    #[instrument(level = "debug", skip_all)]
    pub fn listen_for_updates(&mut self, hardware_tx: Sender<ThreadMessage>, hardware_rx: Receiver<ThreadMessage>) -> Result<(), HeadsetError> {
        info!("successfully started background listening");
        let mut buf = [0u8; 20];
        let mut context: Context = self.get_context_handle(hardware_rx.clone())?;

        loop {
            // test receiving data across threads
            match hardware_rx.try_recv() {
                Ok(resp) => {
                    trace!("new message from gui thread: {:?}", resp);
                    match resp {
                        ThreadMessage::Ready => {
                            match self.send_info_to_gui(hardware_tx.clone()) {
                                Ok(()) => { 
                                    debug!("successfully sent fields to gui thread");
                                    hardware_tx.send(ThreadMessage::Ready).unwrap();
                                },
                                Err(error) => warn!("unexpected error occurred while sending data to gui thread! {}", error)
                            }
                        },
                        _ => ()
                    }
                },
                Err(_) => ()
            }

            match self.handle.read_timeout(&mut buf, 10) {
                Ok(READ_EMPTY) => (),
                Ok(_) => {
                    if let Ok(resp) = Commands::parse(&buf) {
                        info!("{}", resp);

                        // test sending data across threads
                        match hardware_tx.send(ThreadMessage::HeadsetResponse(resp)) {
                            Ok(_) => trace!("sent {:?} message to gui", resp),
                            Err(error) => warn!("error occurred while sending data over threads! {}", error)
                        }

                        match resp {
                            Response::BatteryLevel(_) => self.battery_level = Some(resp),
                            Response::ChargingStatus(_) => self.charging_status = Some(resp),
                            Response::IsActive(_) => self.headset_status = Some(resp),
                            Response::AutoShutdownTime(_) => self.shutdown_time = Some(resp),
                            Response::SidetoneStatus(_) => self.sidetone_status = Some(resp),
                            Response::SidetoneVolume(_) => self.sidetone_volume = Some(resp),
                            Response::NoiseGateStatus(_) => self.noisegate_status = Some(resp),
                            Response::MicrophoneStatus(_) => self.microphone_status = Some(resp),
                        };

                        if resp == Response::IsActive(false) {
                            trace!("device disconnected!")
                        }
                    }

                    context.request_repaint();
                    buf = [0u8; 20];
                },
                Err(error) => {
                    return Err(HeadsetError::HidApiError { msg: error.to_string() })
                }
            }

        }
    }

    // Usable commands for command line arguments
    #[instrument(level = "debug", skip_all)]
    pub fn get_battery(&mut self) -> Result<(), HeadsetError> {
        debug!("querying device");
        execute!(self, battery_level, Commands::GetBatteryLevel);

        Ok(())
    }   

    #[instrument(level = "debug", skip_all)]
    pub fn get_charging_status(&mut self) -> Result<(), HeadsetError> {
        debug!("querying device");
        execute!(self, charging_status, Commands::GetChargingStatus);

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_headset_status(&mut self) -> Result<(), HeadsetError> {
        debug!("querying device");
        execute!(self, headset_status, Commands::GetHeadsetStatus);

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_mic_status(&mut self) -> Result<(), HeadsetError> {
        debug!("querying device");
        execute!(self, microphone_status, Commands::GetMicrophoneStatus);

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_shutdown_time(&mut self) -> Result<(), HeadsetError> {
        debug!("querying device");
        execute!(self, shutdown_time, Commands::GetAutoShutdownTime);

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_sidetone_status(&mut self) -> Result<(), HeadsetError> {
        debug!("querying device");
        execute!(self, sidetone_status, Commands::GetSidetoneStatus);

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_sidetone_volume(&mut self) -> Result<(), HeadsetError> {
        debug!("querying device");
        execute!(self, sidetone_volume, Commands::GetSidetoneVolume);

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_noisegate_status(&mut self) -> Result<(), HeadsetError> {
        debug!("querying device");
        execute!(self, noisegate_status, Commands::GetNoiseGateStatus);

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn set_shutdown_time(&mut self, time: Option<u8>) -> Result<(), HeadsetError> {
        debug!("querying device");
        execute!(self, shutdown_time, Commands::SetAutoShutdownTime, time);

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn set_sidetone_status(&mut self, status: Option<u8>) -> Result<(), HeadsetError> {
        debug!("querying device");
        execute!(self, sidetone_status, Commands::SetSidetoneStatus, status);

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn set_sidetone_volume(&mut self, volume: Option<u8>) -> Result<(), HeadsetError> {
        debug!("querying device");
        execute!(self, sidetone_volume, Commands::SetSidetoneVolume, volume);

        Ok(())
    }

}