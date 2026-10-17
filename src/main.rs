mod hardware;
mod ui;

use hidapi::{HidApi, HidError, HidDevice};
use crossbeam_channel::unbounded;
use eframe::egui::Context;
use tracing::{info, debug, error, warn, trace, Level, subscriber, instrument};
use tracing_subscriber::{FmtSubscriber, EnvFilter, fmt, prelude::*, registry::Registry, filter::{Targets, LevelFilter, Directive}};
use hardware::{Response, HeadsetInfo, Commands, HeadsetError};
use clap::Parser;
use std::thread::{sleep, spawn};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::env;

pub fn sleep_ms(ms: u64) {
    sleep(Duration::from_millis(ms))
}

pub fn sleep_sec(sec: u64) {
    sleep(Duration::from_secs(sec))
}

#[derive(Debug)]
enum ThreadMessage {
    HeadsetResponse(hardware::Response),
    Request(hardware::Commands),
    Ready,
    Set { command: hardware::Commands, param: Option<u8> },
    Context(eframe::egui::Context)
}

#[instrument(level = "debug", skip_all)]
fn init_device() -> Result<HeadsetInfo, HeadsetError> {
    let mut device = HeadsetInfo::new()?;

    device.query_device()?;
    debug!("successfully queried device");

    info!("successfully initialized device!");
    Ok(device)
}

#[instrument(level = "debug", skip_all)]
fn logger_setup() {
    let formatter = fmt::layer()
                    .with_thread_names(true)
                    .with_target(false);

    let environment = EnvFilter::builder().with_default_directive(Level::INFO.into())
                                        .from_env_lossy()
                                        .add_directive("smithay_clipboard=off".parse().unwrap())
                                        .add_directive("calloop_wayland_source=off".parse().unwrap())
                                        .add_directive("calloop=off".parse().unwrap())
                                        .add_directive("winit=off".parse().unwrap());
    
    let level_hint = environment.max_level_hint();

    let subscriber = Registry::default()
                            .with(formatter)
                            .with(environment);

    let logging = subscriber::set_global_default(subscriber);

    match logging {
        Ok(_) => if let Some(level) = level_hint {
            println!("logging enabled");
            println!("disabled scopes: smithay_clipboard, calloop_wayland_source, calloop, winit");
            println!("current verbosity level: {}", level);
            if cfg!(unix) { warn!("unix system detected, device information may be inaccurate") }
        },
        Err(error) => eprintln!("failed to initialize logging: {}", error)
    }
}

#[instrument(level = "debug", skip_all)]
fn main() {
    logger_setup();
    let (hardware_tx, gui_rx) = unbounded::<ThreadMessage>();
    let (gui_tx, hardware_rx) = unbounded::<ThreadMessage>();

    spawn(move || {
        let mut retry_count: u8 = 0;
        let mut retry_sec: u64;

        // Attempt to print device information
        if let Ok(temp) = HeadsetInfo::new() {
            let _ = temp.about_device();
        } else {
            warn!("failed to print device information!")
        }

        'keep_alive: loop {
            trace!("trying to initialize headset...");

            let mut device = match init_device() {
                Ok(device) => {
                    retry_count = 0;
                    device
                },
                Err(error) => {
                    retry_sec = match retry_count {
                        0..=4 => 2,
                        5..=9 => 5,
                        10..=20 => 10,
                        _ => 60
                    };
                    error!("device initialization failed!");
                    error!("reason: {}, attempt: #{}", error, retry_count);
                    trace!("delay until next attempt: {} sec(s)", retry_sec);
                    sleep_sec(retry_sec);
                    retry_count = retry_count.saturating_add(1);

                    continue 'keep_alive;
                }
            };

            info!("device ready!");
            if let Err(error) = device.listen_for_updates(hardware_tx.clone(), hardware_rx.clone()) {
                error!("{} attempting to re-initialize device...", error);
            }

            sleep_sec(2);
        }
    });

    ui::main(gui_tx, gui_rx);
}