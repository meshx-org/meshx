use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

use clap::Parser;
use hcl::eval::{Context, Evaluate, FuncArgs, FuncDef, ParamType};
use hcl::value;
use hcl::BlockLabel;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input BUILD.hcl files
    #[arg(short, long, num_args = 0..)]
    input: Vec<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Dependency {
    #[serde(skip_serializing_if = "Option::is_none")]
    projects: Option<Vec<String>>,
    target: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Target {
    depends_on: Vec<Dependency>,
    executor: String,
    options: serde_json::Value,
    outputs: Vec<String>,
    inputs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Metadata {
    project_deps: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProjectConfiguration {
    name: String,
    root: PathBuf,
    tags: Vec<String>,
    project_type: Option<String>,
    targets: HashMap<String, Target>,
    implicit_dependencies: Vec<String>,
    metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateNodesResult {
    projects: HashMap<String, ProjectConfiguration>,
}

struct TargetContext<'ctx> {
    /// The name of the current target.
    target_name: String,
    target_out_dir: &'ctx str,

    /// Directory for a target's generated files.
    target_gen_dir: &'ctx str,

    project: Rc<RefCell<ProjectConfiguration>>,
}

impl<'ctx> Clone for TargetContext<'ctx> {
    fn clone(&self) -> TargetContext<'ctx> {
        TargetContext {
            target_name: self.target_name.clone(),
            target_out_dir: self.target_out_dir,
            target_gen_dir: self.target_gen_dir,
            project: self.project.clone(),
        }
    }
}

impl<'ctx> TargetContext<'ctx> {
    fn with_target_name<'a: 'ctx>(&self, new_target_name: String) -> Self {
        TargetContext {
            target_name: new_target_name,
            target_out_dir: self.target_out_dir,
            target_gen_dir: self.target_gen_dir,
            project: self.project.clone(),
        }
    }
}

fn build_deps(ctx: &mut TargetContext<'_>, attrs: &HashMap<hcl::Identifier, hcl::Value>) -> Vec<Dependency> {
    attrs
        .get("deps")
        .unwrap_or(&hcl::Value::Array(vec![]))
        .as_array()
        .expect("deps must be an array")
        .into_iter()
        .map(|v| {
            let val = v.as_str().unwrap();
            let target_and_proj: Vec<&str> = val.split(':').collect();

            if target_and_proj.len() == 0 {
                let target = target_and_proj[0];
                Dependency {
                    projects: None,
                    target: format!("build_{target}"),
                }
            } else {
                let proj = target_and_proj[0];
                let target = target_and_proj[1];
                Dependency {
                    projects: if proj == "" {
                        None
                    } else {
                        let mut project = ctx.project.borrow_mut();
                        project.metadata.project_deps.push(proj.to_string());
                        project.implicit_dependencies.push(proj.to_string());
                        Some(vec![proj.to_string()])
                    },
                    target: format!("build_{target}"),
                }
            }
        })
        .collect()
}

fn generated_file(ctx: &mut TargetContext<'_>, contents: String, outputs: Vec<String>) {
    let path: PathBuf = outputs[0].clone().into();
    //fs::create_dir_all(path.parent().unwrap().to_owned()).unwrap();
    //let mut file = fs::File::create(path.clone()).unwrap();
    //file.write_all(contents.as_bytes()).unwrap();
    let mut proj = ctx.project.borrow_mut();

    proj.implicit_dependencies.push("tools/mx-build".to_string());
    proj.targets.insert(
        format!("build_{}", ctx.target_name),
        Target {
            inputs: vec![],
            depends_on: vec![Dependency {
                target: String::from("^build"),
                projects: None,
            }],
            outputs: vec![format!("{{workspaceRoot}}/{}", path.display())],
            executor: String::from("./dist/tools/mx-build:generated_file"),
            options: json!({
                "path": path.display().to_string(),
                "content": contents
            }),
        },
    );
}

// Looks just like a generated_file() target but $outputs is like resource().
fn generated_resource(ctx: &mut TargetContext<'_>, contents: String, output: String) {
    // Select a place to generate the contents at `gn gen` time.
    let file = format!("{}/{}/{output}", ctx.target_gen_dir, ctx.target_name);

    generated_file(ctx, contents, vec![file]);
}

/// Generate meta/package file.
///
/// **Parameters**
///   - applicable_licenses (optional)
///   - package_name (required)
///   - testonly
///   - visibility
fn generate_meta_package<'a>(ctx: &'a mut TargetContext<'_>, package_name: String) {
    let contents = format!("{{\"name\":\"{package_name}\",\"version\":\"0\"}}");
    let output = "meta/package".to_string();
    generated_resource(ctx, contents, output);
}

/// Defines a MeshX package.
/// See: https://fuchsia.dev/fuchsia-src/development/components/build
///
/// MeshX packages are a collection of any number of files (or resources), each
/// with a unique path that is relative to the package's root.
/// Package targets collect resources via their dependencies. These dependencies
/// are typically either:
///
///  * `meshx_component "<target_name>"` targets, which provide their component manifest and
///    other files that the component needs (such as an executable)
///  * other `meshx_package "<target_name>"` targets, declared as `subpackages`
///
/// Packages can be defined as a collection of pairs each representing a file in
/// the package. Each pair consists of the path in the package that is assigned
/// to the file, and a path relative to the build system's output directory where
/// the contents of the file will be sourced from.
/// This mapping is generated at build time, and is known as the package
/// manifest.
///
/// The package name is defined by the target name.
/// Some rules apply to package names.
/// See: https://fuchsia.dev/fuchsia-src/concepts/packages/package_url#package-name
///
/// It's recommended for a package to depend on one or more `fuchsia_component()`
/// targets, and zero or more `subpackages` and/or `renameable_subpackages`.
///
/// Examples:
/// ```hcl
/// meshx_package "my-package" {
///   deps = [
///    ":main_component",
///   ]
///   subpackages = [
///     ":second_package",
///   ]
/// }
/// ```
fn meshx_package<'a>(
    ctx: &'a mut TargetContext<'a>,
    attrs: &HashMap<hcl::Identifier, hcl::Value>,
    root: PathBuf,
    projects: &mut HashMap<String, ProjectConfiguration>,
) {
    let deps = build_deps(ctx, attrs);

    let package_name = ctx.target_name.to_owned();
    let repository = "meshx.co";
    let mini_manifest = format!("{}/{}_manifest", ctx.target_out_dir, ctx.target_name);

    // LINT.IfChange
    let package_out_dir = format!("{}/{}", ctx.target_out_dir, ctx.target_name);
    let package_manifest = format!("{package_out_dir}/package_manifest.json");
    // LINT.ThenChange(//build/packages/exported_fuchsia_package_archive.gni)

    // Generate the "meta/package" file
    let meta_package_target = format!("{}_meta_package", ctx.target_name);

    let mut ctx2 = ctx.with_target_name(meta_package_target);

    generate_meta_package(
        &mut ctx2,
        package_name,
        //forward_variables_from(invoker, ["applicable_licenses", "testonly"]),
        //visibility = [":*"],
        //package_name = package_name,
    );

    ctx.project.borrow_mut().targets.insert(
        format!("build_{}", ctx.target_name),
        Target {
            inputs: vec![],
            outputs: vec![],
            depends_on: deps,
            executor: String::from("nx:noop"),
            options: json!({}),
        },
    );
}

fn generate_component(
    ctx: &mut TargetContext<'_>,
    attrs: &HashMap<hcl::Identifier, hcl::Value>,
    root: PathBuf,
    projects: &mut HashMap<String, ProjectConfiguration>,
) {
    let deps = build_deps(ctx, attrs);

    ctx.project.borrow_mut().targets.insert(
        format!("build_{}", ctx.target_name),
        Target {
            inputs: vec![],
            outputs: vec![],
            depends_on: deps,
            executor: String::from("nx:noop"),
            options: json!({}),
        },
    );
}

/// Declares a MIDL library.
///
/// Supported backends: rust, typescript
fn generate_midl(
    ctx: &mut TargetContext<'_>,
    attrs: &HashMap<hcl::Identifier, hcl::Value>,
    root: PathBuf,
    projects: &mut HashMap<String, ProjectConfiguration>,
) {
    let deps = build_deps(ctx, attrs);

    let enable_rust = attrs
        .get("enable_rust")
        .unwrap_or(&hcl::Value::Bool(true))
        .as_bool()
        .expect("provided default");

    let enable_typescript = attrs
        .get("enable_typescript")
        .unwrap_or(&hcl::Value::Bool(true))
        .as_bool()
        .expect("provided default");

    let name = attrs
        .get("name")
        .unwrap_or(&hcl::Value::String(ctx.target_name.to_string()))
        .as_str()
        .expect("sources must be an array");

    let sources: Vec<&str> = attrs
        .get("sources")
        .expect("sources are required")
        .as_array()
        .expect("sources must be an array")
        .into_iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    let inputs: Vec<String> = sources.iter().map(|src| format!("{{workspaceRoot}}/{}", src)).collect();

    let mut proj = ctx.project.borrow_mut();
    proj.implicit_dependencies.push("tools/nx-midl".to_string());
    proj.targets.insert(
        format!("build_{}", ctx.target_name),
        Target {
            inputs,
            depends_on: deps,
            outputs: vec![format!(
                "{{workspaceRoot}}/dist/{0}/{1}/{1}.midl.json",
                root.display(),
                ctx.target_name
            )],
            executor: String::from("@meshx-org/nx-midl:echo"),
            options: json!({
                "outDir": format!("dist/{0}/{1}/{1}", root.display(), ctx.target_name),
                "midlJson": "./msx.json",
                "srcs": sources
            }),
        },
    );
}

fn glob(args: FuncArgs) -> Result<hcl::Value, String> {
    let pattern_param = &args[0];
    let basedir_param = &args[1];

    let pattern = pattern_param.as_str().unwrap();
    let basedir = basedir_param.as_str().unwrap();

    let walker = globwalker::GlobWalkerBuilder::from_patterns(basedir, &[pattern])
        .build()
        .unwrap()
        .into_iter()
        .filter_map(Result::ok);

    let mut result = vec![];
    for entry in walker {
        result.push(entry.path().display().to_string());
    }

    Ok(hcl::Value::from(result))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut output = vec![];

    for input in args.input.into_iter() {
        let mut projects = HashMap::new();

        let contents = fs::read_to_string(input.clone()).expect("Should have been able to read the file");
        let mut body = hcl::parse(&contents)?;
        let mut ctx = Context::new();
        let root = input.parent().as_ref().unwrap().to_path_buf();

        ctx.declare_var("current", root.display().to_string());
        ctx.declare_var("outdir", "dir");
        ctx.declare_var("var", value!({ example = false }));
        ctx.declare_var("terminal_font_path", "mypath");

        let glob_def = FuncDef::builder()
            .params([ParamType::String, ParamType::String])
            .build(glob);
        ctx.declare_func("glob", glob_def);

        // This will try to evaluate all expressions in `body` and updates it in-place, returning all
        // errors that occurred along the way.
        if let Err(errors) = body.evaluate_in_place(&ctx) {
            eprintln!("{errors}");
        }

        let body = body.evaluate(&ctx)?;

        let mut targets = HashMap::new();

        targets.insert(
            format!("build"),
            Target {
                inputs: vec![],
                outputs: vec![],
                depends_on: vec![Dependency {
                    projects: None,
                    target: "build_*".to_string(),
                }],
                executor: String::from("nx:noop"),
                options: json!({}),
            },
        );

        let default_project = Rc::new(RefCell::new(ProjectConfiguration {
            name: root.clone().display().to_string(),
            root: root.clone(),
            project_type: Some(String::from("library")),
            tags: vec!["lang:midl".to_string()],
            targets,
            implicit_dependencies: vec![],
            metadata: Metadata { project_deps: vec![] },
        }));

        for block in body.clone().into_blocks() {
            let labels = block.labels();
            let attrs: HashMap<_, hcl::Value> = block
                .body
                .attributes()
                .map(|data| (data.key.clone(), data.expr.clone().into()))
                .collect();

            let target_name = labels[0].as_str().to_string();
            let target_out_dir = format!("dist/{}", root.display());
            let target_out_dir = target_out_dir.as_str();

            let target_gen_dir = format!("dist/gen/{}", root.display());
            let target_gen_dir = target_gen_dir.as_str();

            let mut ctx = TargetContext {
                target_name,
                target_out_dir,
                target_gen_dir,
                project: default_project.clone(),
            };

            match block.identifier() {
                "midl" => {
                    generate_midl(&mut ctx, &attrs, root.clone(), &mut projects);
                }
                "component" => {
                    generate_component(&mut ctx, &attrs, root.clone(), &mut projects);
                }
                "meshx_package" => {
                    meshx_package(&mut ctx, &attrs, root.clone(), &mut projects);
                }
                "resource" => {}
                v => panic!("not supported block: {:?}", v),
            }
        }

        let p = (*default_project).borrow();
        let s = p.clone();
        projects.insert(root.clone().display().to_string(), s);

        output.push(json!([input, CreateNodesResult { projects }]));
    }

    serde_json::to_writer(std::io::stdout(), &output)?;

    Ok(())
}
