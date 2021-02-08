
use std::{fs, path::Path};

use crate::error::Result;

pub struct ProcessProbe {
    pid: u32,
}

impl ProcessProbe {
    pub fn new(pid: u32) -> Result<Self> {
        let proc_path = format!("/proc/{}", pid);
        let path = Path::new(&proc_path);
        if !path.exists() {
            let err = std::io::Error::new(std::io::ErrorKind::NotFound, "process does not exists");
            return Err(err.into());
        }
        Ok(Self { pid })
    }

    #[allow(dead_code)]
    pub fn get_stat(&self) -> ProcessStat {
        let content1 = fs::read_to_string(format!("/proc/{}/stat", self.pid)).unwrap();
        let content1: Vec<&str> = content1.trim().split(" ").map(|f| f.trim()).collect();

        let content2 = fs::read_to_string(format!("/proc/{}/status", self.pid)).unwrap();
        let content2: Vec<&str> = content2
            .trim()
            .split("\n")
            .map(|f| f.split(":").last().unwrap().trim())
            .collect();

        ProcessStat {
            pid: content1[0].parse().unwrap(),
            tcomm: content1[1].parse().unwrap(),
            state: content1[2].parse().unwrap(),
            ppid: content1[3].parse().unwrap(),
            pgrp: content1[4].parse().unwrap(),
            sid: content1[5].parse().unwrap(),
            utime: content1[13].parse().unwrap(),
            stime: content1[14].parse().unwrap(),
            cutime: content1[15].parse().unwrap(),
            cstime: content1[16].parse().unwrap(),
            priority: content1[17].parse().unwrap(),
            num_threads: content1[18].parse().unwrap(),
            start_time: content1[20].parse().unwrap(),
            vsize: content1[21].parse().unwrap(),
            rss: content1[22].parse().unwrap(),
            rsslim: content1[23].parse().unwrap(),
            task_cpu: content1[37].parse().unwrap(),
            vm_peak: content2[16].replace("kB", "").trim().parse().unwrap(),
        }
    }

    /// Get the current cpu time usage, user and system
    #[allow(dead_code)]
    fn get_cpu_usage(&self) -> u64 {
        let t = self.get_stat();
        t.utime + t.stime
    }

    /// Get the current memory usage based on resident set memory size
    #[allow(dead_code)]
    fn get_memory_usage(&self) -> u64 {
        self.get_stat().rss
    }

    /// Wait the process to stop and get whole usage status
    pub fn watching(&self) -> ProcessBio {
        let mut status: libc::c_int = 0;
        let mut ru = libc::rusage {
            ru_utime: libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            ru_stime: libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            ru_maxrss: 0,
            ru_ixrss: 0,
            ru_idrss: 0,
            ru_isrss: 0,
            ru_minflt: 0,
            ru_majflt: 0,
            ru_nswap: 0,
            ru_inblock: 0,
            ru_oublock: 0,
            ru_msgsnd: 0,
            ru_msgrcv: 0,
            ru_nsignals: 0,
            ru_nvcsw: 0,
            ru_nivcsw: 0,
        };
        unsafe {
            assert!(
                libc::wait4(
                    self.pid as libc::pid_t,
                    &mut status,
                    libc::WSTOPPED,
                    &mut ru,
                ) >= 0
            );
        }
        ProcessBio {
            status,
            utime: (ru.ru_utime.tv_sec * 1000 + ru.ru_utime.tv_usec / 1000) as u64,
            stime: (ru.ru_stime.tv_sec * 1000 + ru.ru_stime.tv_usec / 1000) as u64,
            maxrss: ru.ru_maxrss as u64,
        }
    }
}

#[derive(Debug)]
pub struct ProcessBio {
    status:i32,
    utime: u64,
    stime: u64,
    maxrss: u64,
}

impl ProcessBio {
    /// Get time usage(ms).
    pub fn get_time_usage(&self)->u64{
        self.utime+self.stime
    }

    pub fn get_status(&self)->i32{
        self.status
    }

    pub fn get_peak_memory(&self)->u64{
        self.maxrss
    }
}

#[derive(Debug)]
pub struct ProcessStat {
    pid: u32,
    tcomm: String,
    state: String,
    ppid: u32,
    pgrp: u32,
    sid: u32,
    utime: u64,
    stime: u64,
    cutime: u64,
    cstime: u64,
    priority: u32,
    num_threads: u32,
    start_time: u64,
    vsize: u64,
    rss: u64,
    rsslim: u64, // bytes
    task_cpu: u32,
    vm_peak: u64, // peak virtual memory size
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;
    #[test]
    fn get_stat() {
        let probe = ProcessProbe::new(process::id()).unwrap();
        assert_eq!(probe.pid,process::id());
    }

    #[test]
    fn wait_process() {}
}
