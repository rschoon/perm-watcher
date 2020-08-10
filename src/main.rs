#[macro_use]
extern crate log;

use chan_signal::Signal;
use clap::{App, Arg};
use env_logger::Env;
use notify::{RecommendedWatcher, Watcher};
use std::error::Error;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

#[derive(Clone)]
struct PermConfig {
    pub perms: u32,
    pub dir_perms: u32,
    pub exact: bool,
}

fn make_mode(current_mode: u32, wanted_mode: u32, exact: bool) -> u32 {
    if exact {
        current_mode & !0o777 | wanted_mode
    } else {
        current_mode | wanted_mode
    }
}

fn chmod(path: &Path, mode: u32) -> Result<(), std::io::Error> {
    let result = unsafe {
        let cstr = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
        libc::chmod(cstr.as_ptr(), mode)
    };

    if result != 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn handle_path_chmod(path: &Path, current_mode: u32, wanted_mode: u32, exact: bool) {
    let new_mode = make_mode(current_mode, wanted_mode, exact);
    if new_mode != current_mode {
        if let Err(err) = chmod(path, new_mode) {
            error!(
                "Couldn't set mode for {} to {:o}: {}",
                path.display(),
                new_mode,
                err
            );
        } else {
            info!(
                "Changed mode for {}: {:o} -> {:o}",
                path.display(),
                current_mode,
                new_mode
            );
        }
    } else {
        debug!(
            "No change in mode for {}: {:o} -> {:o}",
            path.display(),
            current_mode,
            new_mode
        );
    }
}

fn handle_path(path: &Path, config: &PermConfig) {
    debug!("Checking path {}", path.display());

    let md = {
        let md = path.metadata();
        if let Err(err) = md {
            error!("Couldn't get metadata for {}: {}", path.display(), err);
            return;
        }
        md.unwrap()
    };

    if md.is_dir() {
        handle_path_chmod(path, md.mode(), config.dir_perms, config.exact);

        match path.read_dir() {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(ent) => handle_path(&ent.path(), config),
                        Err(err) => {
                            error!("Couldn't read dir entry in {}: {}", path.display(), err)
                        }
                    }
                }
            }
            Err(err) => error!("Couldn't get dir entries for {}: {}", path.display(), err),
        }
    } else {
        handle_path_chmod(path, md.mode(), config.perms, config.exact);
    }
}

fn handle_change(event: &notify::event::Event, config: &PermConfig) {
    for path in event.paths.iter() {
        handle_path(path, config);
    }
}

fn convert_dir_perms(mode: u32) -> u32 {
    let mut mode = mode;
    if mode & 0o400 != 0 {
        mode |= 0o100;
    }
    if mode & 0o040 != 0 {
        mode |= 0o010;
    }
    if mode & 0o004 != 0 {
        mode |= 0o001;
    }
    mode
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("perm-watcher")
        .arg(
            Arg::with_name("perms")
                .long("--perms")
                .short("-p")
                .takes_value(true)
                .default_value("0644"),
        )
        .arg(
            Arg::with_name("dir-perms")
                .long("--dir-perms")
                .short("-d")
                .takes_value(true),
        )
        .arg(Arg::with_name("exact").long("--exact"))
        .arg(
            Arg::with_name("targets")
                .index(1)
                .multiple(true)
                .takes_value(true),
        )
        .get_matches();

    env_logger::from_env(Env::default().default_filter_or("info")).init();

    let perms = matches
        .value_of("perms")
        .ok_or_else(|| "Octal permission required".to_owned())
        .and_then(|v| u32::from_str_radix(v, 8).map_err(|e| e.to_string()))?;
    let dir_perms = match matches.value_of("dir-perms") {
        Some(v) => u32::from_str_radix(v, 8).map_err(|e| e.to_string())?,
        None => convert_dir_perms(perms),
    };
    let exact = matches.is_present("exact");
    let targets = matches.values_of_os("targets").ok_or("Targets required")?;

    let perm_config = PermConfig {
        perms,
        dir_perms,
        exact,
    };

    let perm_config_clone = perm_config.clone();
    let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| match res {
        Ok(res) => handle_change(&res, &perm_config_clone),
        Err(err) => error!("Error during watch: {}", err),
    })?;

    for target in targets {
        let target = PathBuf::from(target);

        info!("Adding watch for {}", target.display());
        watcher.watch(&target, notify::RecursiveMode::Recursive)?;

        handle_path(&target, &perm_config)
    }

    /* Wait forever, or until signal */
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    signal.recv().unwrap();

    info!("Quitting");
    Ok(())
}
