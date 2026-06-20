pub fn run() -> iced::Result {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--setup") => {
            if args.next().is_some() {
                eprintln!("error: too many arguments");
                std::process::exit(1);
            }
            super::run()
        }
        Some("--uninstall") => {
            let Some(root) = args.next() else {
                eprintln!("error: --uninstall requires install root");
                std::process::exit(1);
            };
            if args.next().is_some() {
                eprintln!("error: too many arguments");
                std::process::exit(1);
            }
            super::run_uninstaller(std::path::PathBuf::from(root))
        }
        Some(arg) => {
            eprintln!("error: unknown option: {arg}");
            std::process::exit(1);
        }
        None if running_as_uninstaller() => run_current_root_uninstaller(),
        None => super::run(),
    }
}

fn running_as_uninstaller() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.file_stem().map(|stem| stem.to_os_string()))
        .and_then(|stem| stem.to_str().map(str::to_owned))
        .is_some_and(|stem| stem.eq_ignore_ascii_case("uninstaller"))
}

fn run_current_root_uninstaller() -> iced::Result {
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(error) => {
            eprintln!("error: current exe: {error}");
            std::process::exit(1);
        }
    };
    let root = match k580_ui::install_mode::install_root_from_executable(&exe) {
        Some(root) => root,
        None => {
            eprintln!("error: install root not found");
            std::process::exit(1);
        }
    };
    super::run_uninstaller(root)
}
