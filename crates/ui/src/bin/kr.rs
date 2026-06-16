use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, PartialEq)]
enum CliAction {
    OpenFile(Option<PathBuf>),
    RegisterFileType,
    UnregisterFileType,
    Help,
    Version,
}

fn main() {
    let action = match parse_cli_args(&mut std::env::args().skip(1)) {
        Ok(action) => action,
        Err(message) => {
            eprintln!("error: {message}");
            std::process::exit(1);
        }
    };

    match action {
        CliAction::Help => print_usage(),
        CliAction::Version => print_version(),
        CliAction::RegisterFileType => {
            run_assoc(
                k580_ui::file_assoc::register,
                "Ассоциация .580 зарегистрирована",
            );
        }
        CliAction::UnregisterFileType => {
            run_assoc(k580_ui::file_assoc::unregister, "Ассоциация .580 удалена");
        }
        CliAction::OpenFile(path) => {
            if let Err(error) = spawn_k580(path.as_deref()) {
                eprintln!("error: {error}");
                std::process::exit(1);
            }
        }
    }
}

fn parse_cli_args(args: &mut impl Iterator<Item = String>) -> Result<CliAction, String> {
    let Some(arg) = args.next() else {
        return Ok(CliAction::OpenFile(None));
    };
    match arg.as_str() {
        "--help" | "-h" => Ok(CliAction::Help),
        "--version" | "-V" => Ok(CliAction::Version),
        "--register-file-type" | "-r" => Ok(CliAction::RegisterFileType),
        "--unregister-file-type" | "-u" => Ok(CliAction::UnregisterFileType),
        path => {
            if path.starts_with('-') {
                return Err(format!("unknown option: {path}"));
            }
            if args.next().is_some() {
                return Err("too many arguments".to_owned());
            }
            Ok(CliAction::OpenFile(Some(PathBuf::from(path))))
        }
    }
}

fn print_usage() {
    println!(
        "kr [ПАРАМЕТР] [ФАЙЛ]

Аргументы:
  ФАЙЛ  снимок .580 для открытия

Параметры:
  -h, --help                  Показать справку
  -V, --version               Показать версию
  -r, --register-file-type    Зарегистрировать ассоциацию .580
  -u, --unregister-file-type  Удалить ассоциацию .580"
    );
}

fn print_version() {
    println!("kr {}", env!("CARGO_PKG_VERSION"));
}

fn run_assoc(action: fn() -> Result<(), String>, success: &str) {
    match action() {
        Ok(()) => println!("{success}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

fn spawn_k580(file: Option<&Path>) -> std::io::Result<()> {
    let k580 = match k580_executable() {
        Ok(path) => path,
        Err(error) => {
            #[cfg(debug_assertions)]
            build_k580()?;
            k580_executable().map_err(|_| error)?
        }
    };
    let mut cmd = Command::new(&k580);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(path) = file {
        cmd.arg(path);
    }
    let _child = cmd.spawn()?;
    Ok(())
}

#[cfg(debug_assertions)]
fn build_k580() -> std::io::Result<()> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let status = std::process::Command::new(cargo)
        .args(["build", "-p", "k580-ui", "--bin", "k580"])
        .status()
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("failed to run cargo: {e}"),
            )
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(
            "cargo build -p k580-ui --bin k580 failed",
        ))
    }
}

fn k580_executable() -> std::io::Result<PathBuf> {
    let kr = std::env::current_exe()?;
    let dir = kr
        .parent()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no parent directory"))?;
    let k580 = dir.join(k580_binary_name());
    if k580.is_file() {
        return Ok(k580);
    }

    // Development fallback: `cargo run --bin kr` only builds `kr`, so look
    // for `k580` in the workspace target directory next to the manifest.
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let fallback = manifest_dir
        .join("..")
        .join("..")
        .join("target")
        .join(profile)
        .join(k580_binary_name());
    if fallback.is_file() {
        return Ok(fallback);
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("k580 executable not found at {}", k580.display()),
    ))
}

#[cfg(target_os = "windows")]
fn k580_binary_name() -> &'static str {
    "k580.exe"
}

#[cfg(not(target_os = "windows"))]
fn k580_binary_name() -> &'static str {
    "k580"
}

#[cfg(test)]
mod tests {
    use super::{CliAction, parse_cli_args};
    use std::path::PathBuf;

    #[test]
    fn no_args_opens_empty_file() {
        assert!(matches!(
            parse_cli_args(&mut [].into_iter()),
            Ok(CliAction::OpenFile(None))
        ));
    }

    #[test]
    fn single_path_arg_opens_file() {
        let action = parse_cli_args(&mut ["snapshot.580".to_owned()].into_iter()).unwrap();
        assert_eq!(
            action,
            CliAction::OpenFile(Some(PathBuf::from("snapshot.580")))
        );
    }

    #[test]
    fn known_flags_are_recognized() {
        for flag in [
            "--help",
            "-h",
            "--version",
            "-V",
            "--register-file-type",
            "-r",
            "--unregister-file-type",
            "-u",
        ] {
            assert!(
                parse_cli_args(&mut [flag.to_owned()].into_iter()).is_ok(),
                "flag {flag} should parse"
            );
        }
    }

    #[test]
    fn unknown_flag_errors() {
        assert!(parse_cli_args(&mut ["--unknown".to_owned()].into_iter()).is_err());
    }

    #[test]
    fn too_many_arguments_errors() {
        assert!(parse_cli_args(&mut ["a.580".to_owned(), "b.580".to_owned()].into_iter()).is_err());
    }
}
