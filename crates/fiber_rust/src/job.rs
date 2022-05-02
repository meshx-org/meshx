// Copyright 2017 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//! Type-safe bindings for Zircon jobs.
use crate::{impl_handle_based, ok};
use crate::{object_get_info, ObjectQuery};
use crate::{AsHandleRef, Duration, Handle, HandleBased, HandleRef, Process, Status, Task};
use bitflags::bitflags;
use fiber_sys as sys;
use std::convert::Into;

/// An object representing a Zircon job.
///
/// As essentially a subtype of `Handle`, it can be freely interconverted.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Job(Handle);

impl_handle_based!(Job);

impl Job {
    /// Create a new job as a child of the current job.
    ///
    /// Wraps the
    /// [zx_job_create](https://fuchsia.dev/fuchsia-src/reference/syscalls/job_create.md)
    /// syscall.
    pub fn create_child_job(&self) -> Result<Job, Status> {
        let parent_job_raw = self.raw_handle();
        let mut out = 0;
        let options = 0;
        let status = unsafe { sys::fx_job_create(parent_job_raw, options, &mut out) };
        ok(status)?;
        unsafe { Ok(Job::from(Handle::from_raw(out))) }
    }

    /// Create a new process as a child of the current job.
    ///
    /// On success, returns a handle to the new process and a handle to the
    /// root of the new process's address space.
    ///
    /// Wraps the
    /// [zx_process_create](https://fuchsia.dev/fuchsia-src/reference/syscalls/process_create.md)
    /// syscall.
    pub fn create_child_process(&self, name: &[u8]) -> Result<Process, Status> {
        let parent_job_raw = self.raw_handle();
        let name_ptr = name.as_ptr();
        let name_len = name.len();
        let options = 0;
        let mut process_out = 0;
        let mut vmar_out = 0;
        let status = unsafe {
            sys::fx_process_create(
                parent_job_raw,
                name_ptr,
                name_len,
                options,
                &mut process_out,
                &mut vmar_out,
            )
        };
        ok(status)?;
        unsafe { Ok(Process::from(Handle::from_raw(process_out))) }
    }

    /// Wraps the [zx_job_set_policy](//docs/reference/syscalls/job_set_policy.md) syscall.
    pub fn set_policy(&self, policy: JobPolicy) -> Result<(), Status> {
        match policy {
            JobPolicy::Basic(policy_option, policy_set) => {
                let sys_opt = policy_option.into();
                let sys_topic = sys::FX_JOB_POLICY_BASIC;

                let sys_pol: Vec<sys::fx_policy_basic> = policy_set
                    .into_iter()
                    .map(|(condition, action)| sys::fx_policy_basic {
                        condition: condition.into(),
                        policy: action.into(),
                    })
                    .collect();

                let sys_count = sys_pol.len() as u32;
                let sys_pol_ptr = sys_pol.as_ptr();

                ok(unsafe {
                    // No handles or values are moved as a result of this call (regardless of
                    // success), and the values used here are safely dropped when this function
                    // returns.
                    sys::fx_job_set_policy(
                        self.raw_handle(),
                        sys_opt,
                        sys_topic,
                        sys_pol_ptr as *const u8,
                        sys_count,
                    )
                })
            }
            JobPolicy::TimerSlack(min_slack_duration, default_mode) => {
                let sys_opt = sys::FX_JOB_POLICY_RELATIVE;
                let sys_topic = sys::FX_JOB_POLICY_TIMER_SLACK;

                let sys_pol = sys::fx_policy_timer_slack {
                    min_slack: min_slack_duration.into_nanos(),
                    default_mode: default_mode.into(),
                };

                let sys_count = 1;
                let sys_pol_ptr = &sys_pol as *const sys::fx_policy_timer_slack;

                ok(unsafe {
                    // Requires that `self` contains a currently valid handle.
                    // No handles or values are moved as a result of this call (regardless of
                    // success), and the values used here are safely dropped when this function
                    // returns.
                    sys::fx_job_set_policy(
                        self.raw_handle(),
                        sys_opt,
                        sys_topic,
                        sys_pol_ptr as *const u8,
                        sys_count,
                    )
                })
            }
        }
    }
    /// Wraps the [zx_job_set_critical](//docs/reference/syscalls/job_set_critical.md) syscall.
    pub fn set_critical(&self, opts: JobCriticalOptions, process: &Process) -> Result<(), Status> {
        ok(unsafe { sys::fx_job_set_critical(self.raw_handle(), opts.bits(), process.raw_handle()) })
    }
}

/// Represents the [ZX_JOB_POL_RELATIVE and
/// ZX_JOB_POL_ABSOLUTE](//docs/reference/syscalls/job_set_policy.md) constants
#[derive(Debug, Clone, PartialEq)]
pub enum JobPolicyOption {
    Relative,
    Absolute,
}

impl Into<u32> for JobPolicyOption {
    fn into(self) -> u32 {
        match self {
            JobPolicyOption::Relative => sys::FX_JOB_POLICY_RELATIVE,
            JobPolicyOption::Absolute => sys::FX_JOB_POLICY_ABSOLUTE,
        }
    }
}

/// Holds a timer policy or a basic policy set for
/// [zx_job_set_policy](//docs/reference/syscalls/job_set_policy.md)
#[derive(Debug, Clone, PartialEq)]
pub enum JobPolicy {
    Basic(JobPolicyOption, Vec<(JobCondition, JobAction)>),
    TimerSlack(Duration, JobDefaultTimerMode),
}

/// Represents the [ZX_POL_*](//docs/reference/syscalls/job_set_policy.md) constants
#[derive(Debug, Clone, PartialEq)]
pub enum JobCondition {
    BadHandle,
    WrongObject,
    NewAny,
    NewChannel,
    NewProcess,
}

impl Into<u32> for JobCondition {
    fn into(self) -> u32 {
        match self {
            JobCondition::BadHandle => sys::FX_POLICY_BAD_HANDLE,
            JobCondition::WrongObject => sys::FX_POLICY_WRONG_OBJECT,
            JobCondition::NewAny => sys::FX_POLICY_NEW_ANY,
            JobCondition::NewChannel => sys::FX_POLICY_NEW_CHANNEL,
            JobCondition::NewProcess => sys::FX_POLICY_NEW_PROCESS,
        }
    }
}

/// Represents the [ZX_POL_ACTION_*](//docs/reference/syscalls/job_set_policy.md) constants
#[derive(Debug, Clone, PartialEq)]
pub enum JobAction {
    Allow,
    Deny,
    AllowException,
    DenyException,
    Kill,
}

impl Into<u32> for JobAction {
    fn into(self) -> u32 {
        match self {
            JobAction::Allow => sys::FX_POLICY_ACTION_ALLOW,
            JobAction::Deny => sys::FX_POLICY_ACTION_DENY,
            JobAction::AllowException => sys::FX_POLICY_ACTION_ALLOW_EXCEPTION,
            JobAction::DenyException => sys::FX_POLICY_ACTION_DENY_EXCEPTION,
            JobAction::Kill => sys::FX_POLICY_ACTION_KILL,
        }
    }
}

/// Represents the [ZX_TIMER_SLACK_*](//docs/reference/syscalls/job_set_policy.md) constants
#[derive(Debug, Clone, PartialEq)]
pub enum JobDefaultTimerMode {
    Center,
    Early,
    Late,
}

impl Into<u32> for JobDefaultTimerMode {
    fn into(self) -> u32 {
        match self {
            JobDefaultTimerMode::Center => sys::FX_TIMER_SLACK_CENTER,
            JobDefaultTimerMode::Early => sys::FX_TIMER_SLACK_EARLY,
            JobDefaultTimerMode::Late => sys::FX_TIMER_SLACK_LATE,
        }
    }
}

impl Task for Job {}

bitflags! {
    /// Options that may be used by `Job::set_critical`.
    #[repr(transparent)]
    pub struct JobCriticalOptions: u32 {
        const RETCODE_NONZERO = sys::FX_JOB_CRITICAL_PROCESS_RETCODE_NONZERO;
    }
}

#[cfg(test)]
mod tests {
    // The unit tests are built with a different crate name, but fuchsia_runtime returns a "real"
    // fuchsia_zircon::Job that we need to use.
    use fuchsia_zircon::{
        sys, AsHandleRef, Duration, JobAction, JobCondition, JobCriticalOptions, JobDefaultTimerMode, JobInfo,
        JobPolicy, JobPolicyOption, Signals, Task, Time,
    };
    use std::ffi::CString;

    #[test]
    fn info_default() {
        let job = fuchsia_runtime::job_default();
        let info = job.info().unwrap();
        assert_eq!(
            info,
            JobInfo {
                return_code: 0,
                exited: false,
                kill_on_oom: false,
                debugger_attached: false
            }
        );
    }

    #[test]
    fn runtime_info_default() {
        let job = fuchsia_runtime::job_default();
        let info = job.get_runtime_info().unwrap();
        assert!(info.cpu_time > 0);
        assert!(info.queue_time > 0);
    }

    #[test]
    fn kill_and_info() {
        let default_job = fuchsia_runtime::job_default();
        let job = default_job.create_child_job().expect("Failed to create child job");
        let info = job.info().unwrap();
        assert_eq!(
            info,
            JobInfo {
                return_code: 0,
                exited: false,
                kill_on_oom: false,
                debugger_attached: false
            }
        );
        job.kill().expect("Failed to kill job");
        job.wait_handle(Signals::TASK_TERMINATED, Time::INFINITE).unwrap();
        let info = job.info().unwrap();
        assert_eq!(
            info,
            JobInfo {
                return_code: sys::ZX_TASK_RETCODE_SYSCALL_KILL,
                exited: true,
                kill_on_oom: false,
                debugger_attached: false
            }
        );
    }

    #[test]
    fn create_and_set_policy() {
        let default_job = fuchsia_runtime::job_default();
        let child_job = default_job.create_child_job().expect("failed to create child job");
        child_job
            .set_policy(JobPolicy::Basic(
                JobPolicyOption::Relative,
                vec![
                    (JobCondition::NewChannel, JobAction::Deny),
                    (JobCondition::NewProcess, JobAction::Allow),
                    (JobCondition::BadHandle, JobAction::Kill),
                ],
            ))
            .expect("failed to set job basic policy");
        child_job
            .set_policy(JobPolicy::TimerSlack(
                Duration::from_millis(10),
                JobDefaultTimerMode::Early,
            ))
            .expect("failed to set job timer slack policy");
    }

    #[test]
    fn create_and_set_critical() {
        let default_job = fuchsia_runtime::job_default();
        let child_job = default_job.create_child_job().expect("failed to create child job");
        let binpath = CString::new("/pkg/bin/sleep_forever_util").unwrap();
        let process =
            // Careful not to clone stdio here, or the test runner can hang.
            fdio::spawn(&child_job, fdio::SpawnOptions::DEFAULT_LOADER, &binpath, &[&binpath])
                .expect("Failed to spawn process");
        child_job
            .set_critical(JobCriticalOptions::RETCODE_NONZERO, &process)
            .expect("failed to set critical process for job");
    }
}
