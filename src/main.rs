
#[macro_use]
extern crate log;

use clap::Parser;
use notify::{recommended_watcher, Watcher};
use std::error::Error;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[clap(about, version)]
struct Args {
    /// Unix permissions bits
    #[clap(short, long, value_parser=parse_octal, default_value="0644")]
    perms: u32,

    /// Unix permissions bits (optional directory only)
    #[clap(short, long, value_parser=parse_octal)]
    dir_perms: Option<u32>,

    /// Whether to require exact permissions
    #[clap(long, value_parser)]
    exact: bool,

    /// Target directories to watch
    #[clap(value_parser)]
    targets: Vec<PathBuf>,
}

fn parse_octal(s: &str) -> Result<u32, Box<dyn Error + Send + Sync + 'static>> {
    Ok(u32::from_str_radix(s, 8)?)
}

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
            if matches!(err.kind(), std::io::ErrorKind::NotFound) {
                debug!("Couldn't get metadata for {}: {}", path.display(), err);
            } else {
                error!("Couldn't get metadata for {}: {}", path.display(), err);
            }
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

fn wait_for_terminate() -> i32 {
    use libc::sigset_t;

    let mut retsig = 0;

    unsafe {
        let mut sigset: sigset_t = std::mem::zeroed();
        libc::sigemptyset(&mut sigset);
        libc::sigaddset(&mut sigset, libc::SIGINT);
        libc::sigaddset(&mut sigset, libc::SIGTERM);
        libc::sigprocmask(libc::SIG_BLOCK, &sigset, std::ptr::null_mut::<sigset_t>());
        libc::sigwait(&sigset, &mut retsig);
    }

    retsig
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let dir_perms = args.dir_perms.unwrap_or_else(||convert_dir_perms(args.perms));
    let perm_config = PermConfig {
        perms: args.perms,
        dir_perms,
        exact: args.exact,
    };

    let perm_config_clone = perm_config.clone();
    let mut watcher = recommended_watcher(move |res| match res {
        Ok(res) => handle_change(&res, &perm_config_clone),
        Err(err) => error!("Error during watch: {}", err),
    })?;

    for target in args.targets {
        info!("Adding watch for {}", target.display());
        watcher.watch(&target, notify::RecursiveMode::Recursive)?;

        handle_path(&target, &perm_config)
    }

    /* Run forever, or until signal */
    wait_for_terminate();

    info!("Quitting");
    Ok(())
}
