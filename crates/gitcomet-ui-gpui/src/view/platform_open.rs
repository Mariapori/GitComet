use std::io;
use std::path::Path;

/// Open a URL in the user's default browser.
pub(super) fn open_url(url: &str) -> Result<(), io::Error> {
    if url.trim().is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "URL is empty"));
    }
    open_with_default(url)
}

/// Open a file or directory with the system's default application.
pub(super) fn open_path(path: &Path) -> Result<(), io::Error> {
    open_with_default_os_str(path.as_os_str())
}

/// Open the file manager and select/reveal the given path.
pub(super) fn open_file_location(path: &Path) -> Result<(), io::Error> {
    if path.is_dir() {
        return open_path(path);
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()?;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let mut arg = std::ffi::OsString::from("/select,");
        arg.push(path.as_os_str());
        let _ = std::process::Command::new("explorer.exe")
            .arg(arg)
            .spawn()?;
        return Ok(());
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        let parent = path.parent().unwrap_or(path);
        open_path(parent)
    }

    #[cfg(not(any(
        target_os = "macos",
        target_os = "windows",
        target_os = "linux",
        target_os = "freebsd"
    )))]
    {
        let _ = path;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Opening file locations is not supported on this platform",
        ))
    }
}

fn open_with_default(arg: &str) -> Result<(), io::Error> {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(arg).spawn()?;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(arg)
            .spawn()?;
        return Ok(());
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        match std::process::Command::new("xdg-open").arg(arg).spawn() {
            Ok(_) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                let _ = std::process::Command::new("gio")
                    .args(["open"])
                    .arg(arg)
                    .spawn()?;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    #[cfg(not(any(
        target_os = "macos",
        target_os = "windows",
        target_os = "linux",
        target_os = "freebsd"
    )))]
    {
        let _ = arg;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Opening external resources is not supported on this platform",
        ))
    }
}

fn open_with_default_os_str(arg: &std::ffi::OsStr) -> Result<(), io::Error> {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(arg).spawn()?;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(arg)
            .spawn()?;
        return Ok(());
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        match std::process::Command::new("xdg-open").arg(arg).spawn() {
            Ok(_) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                let _ = std::process::Command::new("gio")
                    .args(["open"])
                    .arg(arg)
                    .spawn()?;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    #[cfg(not(any(
        target_os = "macos",
        target_os = "windows",
        target_os = "linux",
        target_os = "freebsd"
    )))]
    {
        let _ = arg;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Opening files is not supported on this platform",
        ))
    }
}
