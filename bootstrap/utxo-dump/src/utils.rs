use crate::Args;

#[cfg(any(target_os = "macos", target_os = "linux"))]
use libc::{rlimit, setrlimit, RLIMIT_NOFILE};

/// Set file descriptor limit to 16384 for both macOS and Linux
/// This prevents silent data loss when processing large chainstate databases
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub(crate) fn set_unix_rlimit(args: &Args) -> anyhow::Result<()> {
    const TARGET_LIMIT: u64 = 16_384;

    if !args.quiet {
        println!("Setting file descriptor limit to {}", TARGET_LIMIT);
    }

    let lim = rlimit {
        rlim_cur: TARGET_LIMIT, // soft limit
        rlim_max: TARGET_LIMIT, // hard limit
    };

    let ret = unsafe { setrlimit(RLIMIT_NOFILE, &lim) };
    if ret != 0 {
        let error = std::io::Error::last_os_error();
        eprintln!("Failed to set rlimit to {}: {}", TARGET_LIMIT, error);

        let mut current_limit = rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        let get_ret = unsafe { libc::getrlimit(RLIMIT_NOFILE, &mut current_limit) };
        if get_ret == 0 {
            eprintln!(
                "Current limits: soft={}, hard={}",
                current_limit.rlim_cur, current_limit.rlim_max
            );
            if current_limit.rlim_max < TARGET_LIMIT {
                eprintln!("Hard limit ({}) is less than target ({}). You may need to run as root or modify /etc/security/limits.conf", 
                         current_limit.rlim_max, TARGET_LIMIT);
            }
        }

        anyhow::bail!(
            "Failed to set file descriptor limit to {}. This may cause silent data loss!\n\
             Try running: ulimit -n {} before running the tool",
            TARGET_LIMIT,
            TARGET_LIMIT
        );
    } else if !args.quiet {
        println!("Successfully updated RLIMIT_NOFILE to {}", TARGET_LIMIT);
    }

    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub(crate) fn set_unix_rlimit(_args: &Args) -> anyhow::Result<()> {
    eprintln!("Warning: Cannot set rlimit on this platform. Ensure you have sufficient file descriptors available.");
    Ok(())
}
