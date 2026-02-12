#![allow(non_upper_case_globals)]

use fork::fork;
use fork::Fork;
use keyvalues_parser::Vdf;
use procfs;
use smartstring::alias::String as SmartString;
use std::collections::BTreeMap;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::exit;
use std::process::Command;
use std::process::Stdio;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::instrument;
use tracing::trace;
use tracing_unwrap::OptionExt;
use tracing_unwrap::ResultExt;

use home::home_dir;
use once_cell::sync::Lazy;

use crate::daemon::forked_daemon;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SteamApp {
    pub id: u32,
    pub name: SmartString,
    pub path: SmartString,
}

#[derive(Debug)]
pub struct AppProcess {
    pub app: SteamApp,
    pub process: procfs::process::Process,
    pub stat: procfs::process::Stat,
    pub cwd: PathBuf,
    pub exe: PathBuf,
}

impl SteamApp {
    #[instrument]
    pub fn all() -> BTreeMap<u32, SteamApp> {
        ALL_APPS.clone()
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    #[instrument]
    pub fn launch(&self) -> AppProcess {
        if let Some(process) = self.find_process() {
            return process;
        }

        let mut command = Command::new("steam");
        command.arg(format!("steam://rungameid/{}", self.id()));

        info!("Launching {:?} as {:?}", self.name, &command);

        if forked_daemon() {
            let error = command.exec();
            error!("{error:#?}");
            exit(1)
        }

        debug!("Waiting for {:?} to start...", self.name);

        let start = std::time::Instant::now();
        loop {
            if let Some(process) = self.find_process() {
                return process;
            }
            let elapsed = start.elapsed();
            if elapsed > Duration::from_secs(24) {
                error!("No {:?} process after {:?}", self.name, elapsed);
                exit(0);
            }
            sleep(Duration::from_millis(1024));
        }
    }

    #[instrument]
    pub fn find_process(&self) -> Option<AppProcess> {
        trace!("Checking for process...");
        for process in procfs::process::all_processes().unwrap_or_log() {
            let process = process.unwrap_or_log();
            if process.cwd().unwrap_or_default() == self.app_dir() {
                return Some(AppProcess {
                    app: self.clone(),
                    stat: process.stat().unwrap_or_log(),
                    cwd: process.cwd().unwrap_or_log(),
                    exe: process.exe().unwrap_or_log(),
                    process,
                });
            }
        }

        None
    }

    pub fn app_dir(&self) -> PathBuf {
        let mut path = home_dir().unwrap_or_log();
        path.push(".local");
        path.push("share");
        path.push("Steam");
        path.push("steamapps");
        path.push("common");
        path.push(self.path.as_str());
        path
    }

    pub fn saves_dir(&self) -> PathBuf {
        let mut path = home_dir().unwrap_or_log();
        path.push(".local");
        path.push("share");
        path.push(self.path.as_str());
        path.push("Saves");
        path
    }
}

impl AppProcess {
    #[instrument]
    pub fn still_alive(&self) -> bool {
        // if we can't read this file, we assume the process is terminated.
        self.process.stat().is_ok()
    }

    #[instrument]
    pub fn wait_for_exit(&self) {
        debug!("Waiting for {:?} to exit...", self.app.name);
        while self.still_alive() {
            sleep(Duration::from_millis(1024));
        }
    }
}

pub static STEAM_APPS_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = home_dir().unwrap_or_log();
    path.push(".local");
    path.push("share");
    path.push("Steam");
    path.push("steamapps");
    assert!(path.is_dir(), "can't find steamapps");
    path
});

pub static ALL_APPS: Lazy<BTreeMap<u32, SteamApp>> = Lazy::new(|| {
    let mut all_apps = BTreeMap::new();

    for entry in STEAM_APPS_DIR.read_dir().unwrap_or_log() {
        let entry = entry.unwrap_or_log();
        let path = entry.path();
        let file_name = path.file_name().unwrap_or_log().to_string_lossy();
        if !file_name.starts_with("appmanifest_") {
            continue;
        }
        if !file_name.ends_with(".acf") {
            continue;
        }
        let id = u32::from_str(&file_name["appmanifest_".len()..file_name.len() - ".acf".len()])
            .unwrap_or_log();
        let manifest = std::fs::read_to_string(entry.path()).unwrap_or_log();
        let manifest = Vdf::parse(&manifest).unwrap_or_log();
        let manifest = manifest.value.clone();
        let manifest = manifest.get_obj().unwrap_or_log();
        let path = manifest
            .get("installdir")
            .clone()
            .unwrap_or_log()
            .get(0)
            .unwrap_or_log()
            .get_str()
            .unwrap_or_log()
            .to_string()
            .into();
        let name = manifest
            .get("name")
            .clone()
            .unwrap_or_log()
            .get(0)
            .unwrap_or_log()
            .get_str()
            .unwrap_or_log()
            .to_string()
            .into();

        all_apps.insert(id, SteamApp { id, path, name });
    }

    all_apps
});

pub static CELESTE: Lazy<&'static SteamApp> = Lazy::new(|| ALL_APPS.get(&504230).unwrap_or_log());
