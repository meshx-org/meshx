// Copyright 2023 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::rc::Rc;

use cm_config::SecurityPolicy;
use fiber_rust::{self as fx};

use crate::model::token::InstanceRegistry;

use super::runner::BuiltinRunnerFactory;

/// The builtin runner runs components implemented inside component_manager.
///
/// Builtin components are still defined by a declaration. When a component uses
/// the builtin runner, the `type` field in the program block will identify which
/// builtin component to run (e.g `type: "wasm_runner"`).
///
/// When bootstrapping the system, builtin components may be resolved by the builtin URL
/// scheme, e.g. meshx-builtin://#wasm_runner.cm. However, it's entirely possible to resolve
/// a builtin component via other schemes. A component is a builtin component if and only
/// if it uses the builtin runner.
pub struct BuiltinRunner {
    //root_job: fx::Unowned<'static, fx::Job>,
    //task_group: TaskGroup,
    wasm_runner_resources: Rc<WasmRunnerResources>,
}

/// Pure data type holding some resources needed by the WASM runner.
// TODO(https://fxbug.dev/318697539): Most of this should be replaced by
// capabilities in the incoming namespace of the WASM runner component.
pub struct WasmRunnerResources {
    /// Job policy requests in the program block of WASM components will be checked against
    /// the provided security policy.
    pub security_policy: Rc<SecurityPolicy>,
    pub instance_registry: Rc<InstanceRegistry>,
}

impl BuiltinRunner {
    /// Creates a builtin runner with its required resources.
    /// - `task_group`: The tasks associated with the builtin runner.
    pub fn new(wasm_runner_resources: WasmRunnerResources) -> Self {
        //let root_job = meshx_runtime::job_default();
        BuiltinRunner {
            //root_job,
            wasm_runner_resources: Rc::new(wasm_runner_resources),
        }
    }
}

impl BuiltinRunnerFactory for BuiltinRunner {
    fn get_scoped_runner(
        self: Rc<Self>,
        //_checker: ScopedPolicyChecker,
        //open_request: OpenRequest<'_>,
    ) -> Result<(), fx::Status> {
        /*open_request.open_service(endpoint(move |_scope, server_end| {
            let runner = self.clone();
            let mut stream = fcrunner::ComponentRunnerRequestStream::from_channel(server_end);
            runner.clone().task_group.spawn(async move {
                while let Ok(Some(request)) = stream.try_next().await {
                    let fcrunner::ComponentRunnerRequest::Start { start_info, controller, .. } =
                        request;
                    match runner.clone().start(start_info) {
                        Ok((program, on_exit)) => {
                            let controller =
                                Controller::new(program, controller.into_stream().unwrap());
                            runner.task_group.spawn(controller.serve(on_exit));
                        }
                        Err(err) => {
                            warn!("Builtin runner failed to run component: {err}");
                            let _ = controller.close_with_epitaph(err.into());
                        }
                    }
                }
            });
        }))*/
        Ok(())
    }
}