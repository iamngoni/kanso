//! Install and operate the platform service that runs `kanso serve`.
//!
//! The service never removes the data directory. On Unix the service files
//! are rendered locally and then handed to systemd or launchd.

use std::path::{Path, PathBuf};
use std::process::Command as Process;

use anyhow::{Context as _, Result, bail};

const HELP: &str = "\
Usage: kanso service <install|uninstall|start|stop|status> [--user <name>] [--data-dir <dir>]

Runs `kanso serve` at startup and restarts it after a failure. Uninstall
removes the service file, never the library or its secrets.
";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Install,
    Uninstall,
    Start,
    Stop,
    Status,
}

#[derive(Debug, PartialEq, Eq)]
struct Options {
    action: Action,
    user: Option<String>,
    data_dir: Option<PathBuf>,
}

fn parse(args: &[String]) -> Result<Options> {
    let Some(action) = args.first().map(String::as_str) else {
        bail!("{HELP}");
    };
    let action = match action {
        "install" => Action::Install,
        "uninstall" => Action::Uninstall,
        "start" => Action::Start,
        "stop" => Action::Stop,
        "status" => Action::Status,
        "help" | "--help" | "-h" => bail!("{HELP}"),
        other => bail!("unknown service command `{other}`\n\n{HELP}"),
    };
    let mut options = Options {
        action,
        user: None,
        data_dir: None,
    };
    let mut args = args.iter().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--user" => {
                let value = args
                    .next()
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| anyhow::anyhow!("--user needs a user name"))?;
                options.user = Some(value.clone());
            }
            "--data-dir" => {
                let value = args
                    .next()
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| anyhow::anyhow!("--data-dir needs a path"))?;
                options.data_dir = Some(PathBuf::from(value));
            }
            other => bail!("service doesn't take `{other}`\n\n{HELP}"),
        }
    }
    Ok(options)
}

pub fn run_cli(args: &[String]) -> Result<()> {
    if args.is_empty()
        || matches!(
            args.first().map(String::as_str),
            Some("help" | "--help" | "-h")
        )
    {
        println!("{HELP}");
        return Ok(());
    }
    let options = parse(args)?;
    #[cfg(target_os = "linux")]
    return linux::run(options);
    #[cfg(target_os = "macos")]
    return macos::run(options);
    #[cfg(target_os = "windows")]
    {
        return windows::run(options);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = options;
        bail!("kanso service: this platform has no supported service manager");
    }
}

#[cfg(windows)]
pub(crate) fn run_windows_service(data_dir: PathBuf) -> Result<()> {
    windows::run_service(data_dir)
}

#[cfg(windows)]
pub(crate) fn windows_stop_flag() -> Option<std::sync::Arc<std::sync::atomic::AtomicBool>> {
    windows::stop_flag()
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;

    const UNIT: &str = include_str!("../../../assets/linux/kanso.service");
    const SYSTEM_UNIT: &str = "/etc/systemd/system/kanso.service";

    fn systemctl(user: bool, args: &[&str]) -> Result<()> {
        let mut command = Process::new("systemctl");
        if user {
            command.arg("--user");
        }
        let status = command.args(args).status().context("running systemctl")?;
        if !status.success() {
            bail!("systemctl {} exited with {status}", args.join(" "));
        }
        Ok(())
    }

    fn user_service_path() -> Result<PathBuf> {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .context("HOME is not set")?;
        Ok(home.join(".config/systemd/user/kanso.service"))
    }

    fn render(user: Option<&str>, data_dir: &Path) -> String {
        let directory = systemd_escape(&data_dir.display().to_string());
        let unit = if let Some(user) = user {
            UNIT.replace("User=kanso", &format!("User={user}"))
                .replace("Group=kanso", &format!("Group={user}"))
        } else {
            UNIT.replace("User=kanso\n", "")
                .replace("Group=kanso\n", "")
                .replace("ProtectHome=yes\n", "")
        };
        unit.replace("StateDirectory=kanso", "")
            .replace(
                "Environment=KANSO_DATA_DIR=/var/lib/kanso",
                &format!("Environment=KANSO_DATA_DIR={directory}"),
            )
            .replace(
                "ReadWritePaths=/var/lib/kanso",
                &format!("ReadWritePaths={directory}"),
            )
    }

    fn systemd_escape(path: &str) -> String {
        path.chars()
            .map(|ch| match ch {
                '\\' => "\\x5c".to_owned(),
                ' ' => "\\x20".to_owned(),
                '\t' => "\\x09".to_owned(),
                '%' => "%%".to_owned(),
                ch => ch.to_string(),
            })
            .collect()
    }

    fn write_unit(path: &Path, contents: &str) -> Result<()> {
        let parent = path.parent().context("service unit has no parent folder")?;
        std::fs::create_dir_all(parent).context("creating the systemd unit folder")?;
        std::fs::write(path, contents).context("writing the systemd unit")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o644))?;
        }
        Ok(())
    }

    pub(super) fn run(options: Options) -> Result<()> {
        let system = unsafe { libc::geteuid() } == 0;
        let user = options
            .user
            .or_else(|| (!system).then(|| std::env::var("USER").ok()).flatten());
        let data_dir = options.data_dir.unwrap_or_else(|| {
            if system {
                PathBuf::from("/var/lib/kanso")
            } else {
                super::super::paths::default_data_dir()
            }
        });
        let unit_path = if system {
            PathBuf::from(SYSTEM_UNIT)
        } else {
            user_service_path()?
        };
        match options.action {
            Action::Install => {
                if system && user.is_none() {
                    bail!("kanso service: pass --user <name> when installing as root");
                }
                if system {
                    let user = user.as_deref().unwrap();
                    let exists = Process::new("id").args(["-u", user]).status()?.success();
                    if !exists {
                        let status = Process::new("useradd")
                            .args(["--system", "--home-dir"])
                            .arg(&data_dir)
                            .args(["--shell", "/usr/sbin/nologin", user])
                            .status()
                            .context("creating the service user")?;
                        if !status.success() {
                            bail!("kanso service: useradd exited with {status}");
                        }
                    }
                    std::fs::create_dir_all(&data_dir)?;
                    let status = Process::new("chown")
                        .arg(format!("{user}:{user}"))
                        .arg(&data_dir)
                        .status()?;
                    if !status.success() {
                        bail!("kanso service: chown exited with {status}");
                    }
                    let status = Process::new("chmod")
                        .args(["700"])
                        .arg(&data_dir)
                        .status()?;
                    if !status.success() {
                        bail!("kanso service: chmod exited with {status}");
                    }
                } else {
                    std::fs::create_dir_all(&data_dir)?;
                    use std::os::unix::fs::PermissionsExt as _;
                    std::fs::set_permissions(&data_dir, std::fs::Permissions::from_mode(0o700))?;
                }
                write_unit(
                    &unit_path,
                    &render(
                        system.then(|| user.as_deref().unwrap_or("kanso")),
                        &data_dir,
                    ),
                )?;
                systemctl(!system, &["daemon-reload"])?;
                systemctl(!system, &["enable", "--now", "kanso.service"])?;
                println!("Installed {} and started it.", unit_path.display());
                if system {
                    println!(
                        "To sign in while the service runs, use `sudo -u {} kanso account sign-in --data-dir {}`; it uses the service's control channel.",
                        user.as_deref().unwrap(),
                        data_dir.display()
                    );
                } else {
                    println!(
                        "For a user service on a server, enable lingering with `loginctl enable-linger`. To sign in while it runs, use `kanso account sign-in --data-dir {}`; it uses the service's control channel.",
                        data_dir.display()
                    );
                }
                Ok(())
            }
            Action::Uninstall => {
                let _ = systemctl(!system, &["disable", "--now", "kanso.service"]);
                let _ = std::fs::remove_file(&unit_path);
                systemctl(!system, &["daemon-reload"])?;
                println!(
                    "Removed {}. The library at {} is still there.",
                    unit_path.display(),
                    data_dir.display()
                );
                Ok(())
            }
            Action::Start => systemctl(!system, &["start", "kanso.service"]),
            Action::Stop => systemctl(!system, &["stop", "kanso.service"]),
            Action::Status => systemctl(!system, &["status", "kanso.service"]),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn system_unit_is_bound_to_the_service_user_and_data_folder() {
            let unit = render(Some("kanso"), Path::new("/var/lib/kanso"));
            assert!(unit.contains("ExecStart=/usr/local/bin/kanso serve"));
            assert!(unit.contains("User=kanso\nGroup=kanso"));
            assert!(unit.contains("Environment=KANSO_DATA_DIR=/var/lib/kanso"));
            assert!(unit.contains("ReadWritePaths=/var/lib/kanso"));
            assert!(unit.contains("UMask=0077"));
        }

        #[test]
        fn a_custom_data_folder_is_used_in_both_environment_and_write_allowlist() {
            let unit = render(Some("notes"), Path::new("/srv/my kanso"));
            assert!(unit.contains("User=notes\nGroup=notes"));
            assert!(unit.contains("Environment=KANSO_DATA_DIR=/srv/my\\x20kanso"));
            assert!(unit.contains("ReadWritePaths=/srv/my\\x20kanso"));
            assert!(!unit.contains("StateDirectory=kanso"));
        }

        #[test]
        fn systemd_paths_escape_spaces_backslashes_and_specifiers() {
            assert_eq!(
                systemd_escape("/srv/my kanso\\box%"),
                "/srv/my\\x20kanso\\x5cbox%%"
            );
        }

        #[test]
        fn user_units_can_read_their_home_data_folder() {
            let unit = render(None, Path::new("/home/ngoni/.local/share/kanso"));
            assert!(!unit.contains("User=kanso"));
            assert!(!unit.contains("ProtectHome=yes"));
            assert!(unit.contains("ReadWritePaths=/home/ngoni/.local/share/kanso"));
        }

        proptest! {
            #[test]
            fn prop_systemd_escaping_never_leaves_path_separators_ambiguous(path in "[a-zA-Z0-9 /%\\\\\\t]{0,80}") {
                let escaped = systemd_escape(&path);
                prop_assert!(!escaped.contains(' '));
                prop_assert!(!escaped.contains('\t'));
                prop_assert!(!escaped.contains('%') || escaped.contains("%%"));
            }
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    const LABEL: &str = "app.kanso.serve";

    fn xml(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }

    fn render(exe: &Path, data_dir: &Path, log: &Path) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\"><dict>\n<key>Label</key><string>{LABEL}</string>\n<key>ProgramArguments</key><array><string>{}</string><string>serve</string></array>\n<key>EnvironmentVariables</key><dict><key>KANSO_DATA_DIR</key><string>{}</string></dict>\n<key>RunAtLoad</key><true/><key>KeepAlive</key><true/>\n<key>StandardOutPath</key><string>{}</string><key>StandardErrorPath</key><string>{}</string>\n</dict></plist>\n",
            xml(&exe.display().to_string()),
            xml(&data_dir.display().to_string()),
            xml(&log.display().to_string()),
            xml(&log.display().to_string())
        )
    }

    pub(super) fn run(options: Options) -> Result<()> {
        let home = super::super::paths::home();
        let data_dir = options
            .data_dir
            .unwrap_or_else(super::super::paths::default_data_dir);
        let path = home.join("Library/LaunchAgents/app.kanso.serve.plist");
        let exe = std::env::current_exe().context("finding Kanso's executable")?;
        let log = home.join("Library/Logs/Kanso/serve.log");
        let uid = Process::new("id").arg("-u").output()?.stdout;
        let uid = String::from_utf8_lossy(&uid).trim().to_owned();
        let domain = format!("gui/{uid}");
        match options.action {
            Action::Install => {
                std::fs::create_dir_all(path.parent().unwrap())?;
                std::fs::create_dir_all(log.parent().unwrap())?;
                std::fs::create_dir_all(&data_dir)?;
                use std::os::unix::fs::PermissionsExt as _;
                std::fs::set_permissions(&data_dir, std::fs::Permissions::from_mode(0o700))?;
                std::fs::write(&path, render(&exe, &data_dir, &log))?;
                let status = Process::new("launchctl")
                    .args(["bootstrap", &domain])
                    .arg(&path)
                    .status()?;
                if !status.success() {
                    bail!("launchctl bootstrap exited with {status}");
                }
                println!("Installed {} and started it.", path.display());
            }
            Action::Uninstall => {
                let _ = Process::new("launchctl")
                    .args(["bootout", &domain])
                    .arg(&path)
                    .status();
                let _ = std::fs::remove_file(&path);
                println!(
                    "Removed {}. The library at {} is still there.",
                    path.display(),
                    data_dir.display()
                );
            }
            Action::Start => {
                let label = format!("{domain}/{LABEL}");
                let status = Process::new("launchctl")
                    .args(["kickstart", &label])
                    .status()?;
                if !status.success() {
                    bail!("launchctl kickstart exited with {status}");
                }
            }
            Action::Stop => {
                let label = format!("{domain}/{LABEL}");
                let status = Process::new("launchctl")
                    .args(["kill", "SIGTERM", &label])
                    .status()?;
                if !status.success() {
                    bail!("launchctl kill exited with {status}");
                }
            }
            Action::Status => {
                let label = format!("{domain}/{LABEL}");
                let status = Process::new("launchctl").args(["print", &label]).status()?;
                if !status.success() {
                    bail!("launchctl print exited with {status}");
                }
            }
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn launch_agent_runs_at_login_and_keeps_a_log() {
            let plist = render(
                Path::new("/Applications/Kanso.app/Contents/MacOS/kanso"),
                Path::new("/Users/a/Library/Application Support/Kanso"),
                Path::new("/Users/a/Library/Logs/Kanso/serve.log"),
            );
            assert!(plist.contains("<key>KeepAlive</key><true/>"));
            assert!(plist.contains("<key>RunAtLoad</key><true/>"));
            assert!(plist.contains("<string>serve</string>"));
            assert!(plist.contains("StandardErrorPath"));
        }

        #[test]
        fn plist_values_escape_markup() {
            assert_eq!(xml("a&b<c>"), "a&amp;b&lt;c&gt;");
        }
    }
}

#[cfg(windows)]
mod windows {
    use super::*;
    use std::ffi::OsString;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, OnceLock};
    use std::time::Duration;

    use windows_service::service::{
        ServiceAccess, ServiceAction, ServiceActionType, ServiceControl, ServiceControlAccept,
        ServiceErrorControl, ServiceExitCode, ServiceFailureActions, ServiceFailureResetPeriod,
        ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
    };
    use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
    use windows_service::service_dispatcher;
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    const SERVICE_NAME: &str = "Kanso";
    static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();
    static STOP: OnceLock<Arc<AtomicBool>> = OnceLock::new();

    fn manager(access: ServiceManagerAccess) -> Result<ServiceManager> {
        ServiceManager::local_computer(None::<&str>, access)
            .context("opening the Windows service manager")
    }

    fn account_name() -> Result<OsString> {
        let user = std::env::var("USERNAME").context("USERNAME is not set")?;
        let domain = std::env::var("USERDOMAIN").unwrap_or_else(|_| ".".into());
        Ok(format!("{domain}\\{user}").into())
    }

    fn service_info(
        exe: PathBuf,
        data_dir: &Path,
        account: OsString,
        password: OsString,
    ) -> ServiceInfo {
        ServiceInfo {
            name: SERVICE_NAME.into(),
            display_name: "Kanso local API and sync".into(),
            service_type: ServiceType::OWN_PROCESS,
            start_type: ServiceStartType::AutoStart,
            error_control: ServiceErrorControl::Normal,
            executable_path: exe,
            launch_arguments: vec![
                "serve".into(),
                "--as-service".into(),
                "--data-dir".into(),
                data_dir.as_os_str().to_owned(),
            ],
            dependencies: vec![],
            account_name: Some(account),
            account_password: Some(password),
        }
    }

    fn open_service(access: ServiceAccess) -> Result<windows_service::service::Service> {
        manager(ServiceManagerAccess::CONNECT)?
            .open_service(SERVICE_NAME, access)
            .context("opening the Kanso service")
    }

    pub(super) fn run(options: Options) -> Result<()> {
        let data_dir = options
            .data_dir
            .unwrap_or_else(super::super::paths::default_data_dir);
        match options.action {
            Action::Install => {
                let exe = std::env::current_exe().context("finding Kanso's executable")?;
                // no-prop-test: the password prompt is an OS terminal adapter.
                let password = rpassword::prompt_password("Windows service account password: ")?;
                let password = OsString::from(password);
                let account = account_name()?;
                let service =
                    manager(ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE)?
                        .create_service(
                            &service_info(exe, &data_dir, account, password),
                            ServiceAccess::CHANGE_CONFIG | ServiceAccess::START,
                        )
                        .context("creating the Kanso service")?;
                service.set_description("Kanso local API and sync")?;
                service.update_failure_actions(ServiceFailureActions {
                    reset_period: ServiceFailureResetPeriod::After(Duration::from_secs(86_400)),
                    reboot_msg: None,
                    command: None,
                    actions: Some(vec![
                        ServiceAction {
                            action_type: ServiceActionType::Restart,
                            delay: Duration::from_secs(5),
                        },
                        ServiceAction {
                            action_type: ServiceActionType::Restart,
                            delay: Duration::from_secs(30),
                        },
                        ServiceAction {
                            action_type: ServiceActionType::Restart,
                            delay: Duration::from_secs(60),
                        },
                    ]),
                })?;
                std::fs::create_dir_all(&data_dir)?;
                let status = std::fs::metadata(&data_dir)?;
                if !status.is_dir() {
                    bail!("kanso service: the data path is not a directory");
                }
                let acl = Process::new("icacls")
                    .arg(&data_dir)
                    .args(["/inheritance:r", "/grant:r"])
                    .arg(format!("{}:(OI)(CI)F", account_name()?.to_string_lossy()))
                    .status()
                    .context("setting owner-only permissions on the service data folder")?;
                if !acl.success() {
                    bail!("kanso service: icacls exited with {acl}");
                }
                service
                    .start::<OsString>(&[])
                    .context("starting the Kanso service")?;
                println!(
                    "Installed and started the Kanso service. To sign in while it runs, use `kanso account sign-in --data-dir {}`; it uses the service's control channel.",
                    data_dir.display()
                );
            }
            Action::Uninstall => {
                let service = open_service(
                    ServiceAccess::STOP | ServiceAccess::DELETE | ServiceAccess::QUERY_STATUS,
                )?;
                let _ = service.stop();
                service.delete().context("removing the Kanso service")?;
                println!(
                    "Removed the Kanso service. The library at {} is still there.",
                    data_dir.display()
                );
            }
            Action::Start => open_service(ServiceAccess::START)?
                .start::<OsString>(&[])
                .context("starting the Kanso service")?,
            Action::Stop => {
                let _ = open_service(ServiceAccess::STOP)?
                    .stop()
                    .context("stopping the Kanso service")?;
            }
            Action::Status => {
                let status = open_service(ServiceAccess::QUERY_STATUS)?
                    .query_status()
                    .context("reading Kanso service status")?;
                println!("Kanso service: {:?}", status.current_state);
            }
        }
        Ok(())
    }

    pub(crate) fn run_service(data_dir: PathBuf) -> Result<()> {
        let _ = DATA_DIR.set(data_dir);
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)
            .context("registering Kanso with the Windows service manager")?;
        Ok(())
    }

    pub(crate) fn stop_flag() -> Option<Arc<AtomicBool>> {
        STOP.get().cloned()
    }

    // no-prop-test: SCM calls this native entry point; state transitions are covered by the service runner's adapter boundary.
    windows_service::define_windows_service!(ffi_service_main, service_main);

    fn service_main(_arguments: Vec<OsString>) {
        if let Err(err) = run_service_main() {
            eprintln!("kanso service: {err:#}");
        }
    }

    fn run_service_main() -> Result<()> {
        let stop = Arc::new(AtomicBool::new(false));
        STOP.set(stop.clone()).ok();
        let handler_stop = stop.clone();
        let handler = move |event| match event {
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            ServiceControl::Stop | ServiceControl::Shutdown => {
                handler_stop.store(true, Ordering::Relaxed);
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        };
        let status_handle = service_control_handler::register(SERVICE_NAME, handler)?;
        status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::StartPending,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 1,
            wait_hint: Duration::from_secs(5),
            process_id: None,
        })?;
        status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;
        let data_dir = DATA_DIR
            .get()
            .cloned()
            .unwrap_or_else(super::super::paths::default_data_dir);
        let served = crate::serve_cli::run_cli(data_dir, &[]);
        let stopped = status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        });
        served?;
        stopped?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn parses_a_service_action_and_its_options() {
        assert_eq!(
            parse(&["install".into(), "--user".into(), "kanso".into()]).unwrap(),
            Options {
                action: Action::Install,
                user: Some("kanso".into()),
                data_dir: None,
            }
        );
    }

    proptest! {
        #[test]
        fn unknown_flags_are_refused(flag in "-[a-z]{1,12}") {
            prop_assert!(parse(&["install".into(), flag]).is_err());
        }
    }
}
