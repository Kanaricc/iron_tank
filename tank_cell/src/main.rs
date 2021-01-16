use std::{collections::HashSet, ptr::null};

use clap::{App, Arg};
use libc::*;
use seccomp_sys::*;
use std::path::Path;

fn main() {
    let cmd = App::new("Code Loader")
        .version("0.1.0")
        .author("Kanari <iovo7c@gmail.com>")
        .about("Limit loader")
        .arg(
            Arg::with_name("memory_limit")
                .long("memory_limit")
                .short("m")
                .help("set memory limit(MB) for code")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("time_limit")
                .long("time_limit")
                .short("t")
                .help("set time limit(s) for code")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("path")
                .index(1)
                .help("execution path")
                .required(true),
        )
        .arg(
            Arg::with_name("permission")
                .long("permission")
                .short("p")
                .multiple(true)
                .help("set permission for code")
                .takes_value(true)
                .default_value("minimum"),
        )
        .arg(
            Arg::with_name("raw")
                .multiple(true)
                .last(true)
                .help("arguments for code"),
        )
        .get_matches();

    // check if path exists
    let path = Path::new(cmd.value_of("path").unwrap());
    if !path.exists() {
        println!("cell: path does not exists");
        std::process::exit(1);
    }
    let file_name = path.file_name().unwrap().to_str().unwrap();

    // construct parameters passed to exec
    let mut raw_params: Vec<*const i8> = cmd
        .values_of("raw")
        .unwrap_or_default()
        .map(|f| f.as_ptr() as *const i8)
        .collect();
    raw_params.insert(0, file_name.as_ptr() as *const i8);
    raw_params.push(null::<i8>());

    // set memory and time limit
    if let Some(memory_limit) = cmd.value_of("memory_limit") {
        let memory_limit = memory_limit.trim().parse::<u64>().unwrap();
        set_memory_limit(memory_limit);
    }
    if let Some(time_limit) = cmd.value_of("time_limit") {
        let time_limit = time_limit.trim().parse::<u64>().unwrap();
        set_time_limit(time_limit);
    }

    let raw_params = raw_params.as_ptr();
    let _act_bannd = SCMP_ACT_ERRNO(998244353); // a more soft way to block operation
    unsafe {
        let context = seccomp_init(SCMP_ACT_KILL);

        // load minimum rules which make sure the simplest code can run without problem
        let rules: HashSet<&str> = cmd.values_of("permission").unwrap().map(|f|f.trim()).collect();
        if rules.contains("minimum") {
            load_minimum_rules(context);
        }
        if rules.contains("io") {
            load_io_rules(context);
        }

        // rule for execve allowing only being used by us
        let exe = path.to_str().unwrap().as_ptr() as *const i8;
        seccomp_rule_add(
            context,
            SCMP_ACT_ALLOW,
            SYS_execve as i32,
            1,
            scmp_arg_cmp {
                arg: 0,
                op: scmp_compare::SCMP_CMP_EQ,
                datum_a: exe as u64,
                datum_b: 0,
            },
        );

        assert!(seccomp_load(context) == 0);

        libc::execvp(exe, raw_params);
    }
}

unsafe fn allow_syscall(ctx: *mut c_void, ids: Vec<i64>) {
    for id in ids {
        assert!(seccomp_rule_add(ctx, SCMP_ACT_ALLOW, id as i32, 0) == 0);
    }
}

unsafe fn load_minimum_rules(ctx: *mut c_void) {
    allow_syscall(
        ctx,
        vec![
            SYS_read,
            SYS_write,
            // SYS_open,
            SYS_close,
            SYS_stat,
            SYS_fstat,
            SYS_mmap,
            SYS_mprotect,
            SYS_munmap,
            SYS_brk,
            SYS_pread64,
            SYS_pwrite64,
            SYS_access,
            // SYS_execve,
            SYS_exit,
            SYS_arch_prctl,
            SYS_exit_group,
            // SYS_openat,
        ],
    );

    // rule for openat allowing read only
    seccomp_rule_add(
        ctx,
        SCMP_ACT_ALLOW,
        SYS_openat as i32,
        1,
        scmp_arg_cmp {
            arg: 2,
            op: scmp_compare::SCMP_CMP_EQ,
            datum_a: (libc::O_RDONLY | libc::O_CLOEXEC) as u64,
            datum_b: 0,
        },
    );

    // rule for open allowing read only
    seccomp_rule_add(
        ctx,
        SCMP_ACT_ALLOW,
        SYS_open as i32,
        1,
        scmp_arg_cmp {
            arg: 1,
            op: scmp_compare::SCMP_CMP_EQ,
            datum_a: (libc::O_RDONLY | libc::O_CLOEXEC) as u64,
            datum_b: 0,
        },
    );
}

unsafe fn load_io_rules(ctx: *mut c_void) {
    allow_syscall(ctx, vec![SYS_openat, SYS_open]);
}

fn set_memory_limit(lim: u64) {
    let ctx = rlimit64 {
        rlim_cur: lim << 10 << 10 << 1,
        rlim_max: lim << 10 << 10 << 1,
    };
    let ctx: *const rlimit64 = &ctx;
    unsafe {
        assert!(setrlimit64(RLIMIT_AS, ctx) == 0);
    }
}
fn set_time_limit(lim: u64) {
    let ctx = rlimit64 {
        rlim_cur: (lim + 1000) / 1000,
        rlim_max: (lim + 1000) / 1000,
    };
    let ctx: *const rlimit64 = &ctx;
    unsafe {
        assert!(setrlimit64(RLIMIT_CPU, ctx) == 0);
    }
}
