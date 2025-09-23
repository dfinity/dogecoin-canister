use crate::Args;
use libc::{rlimit, setrlimit, RLIMIT_NOFILE};

#[cfg(target_os = "macos")]
pub(crate) fn set_macos_rlimit(args: &Args) -> anyhow::Result<()> {
    if !args.quiet {
        println!("Setting rlimit to 4096");
    }

    let lim = rlimit {
        rlim_cur: 4_096, // soft limit
        rlim_max: 4_096, // hard limit
    };

    let ret = unsafe { setrlimit(RLIMIT_NOFILE, &lim) };
    if ret != 0 {
        eprintln!("Failed to set rlimit: {}", std::io::Error::last_os_error());
    } else {
        println!("Successfully updated RLIMIT_NOFILE to 4096");
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn set_macos_rlimit(_args: &Args) -> anyhow::Result<()> {
    Ok(())
}