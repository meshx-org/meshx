use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fmt::format;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use clap::Parser;
use hcl::eval::{Context, Evaluate, FuncArgs, FuncDef, ParamType};
use hcl::value;
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
    inputs: Vec<Input>,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
enum Input {
    File(String),
    DependentTasks { dependent_tasks_output_files: String },
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
    tags: HashSet<String>,
    project_type: Option<String>,
    targets: HashMap<String, Target>,
    implicit_dependencies: HashSet<String>,
    metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateNodesResult {
    projects: HashMap<String, ProjectConfiguration>,
}

struct TargetContext<'ctx> {
    /// Name of the topmost target
    root_target_name: &'ctx str,

    /// The name of the current target.
    target_name: String,
    target_out_dir: &'ctx str,

    root_build_dir: &'ctx str,

    /// Directory for a target's generated files.
    target_gen_dir: &'ctx str,

    project: Rc<RefCell<ProjectConfiguration>>,
}

impl<'ctx> TargetContext<'ctx> {
    fn with_target_name<'a: 'ctx>(&self, new_target_name: String) -> Self {
        TargetContext {
            root_target_name: self.root_target_name,
            target_name: new_target_name,
            target_out_dir: self.target_out_dir,
            target_gen_dir: self.target_gen_dir,
            root_build_dir: self.root_build_dir,
            project: self.project.clone(),
        }
    }
}

fn to_dep(ctx: &mut TargetContext<'_>, label: &str) -> Dependency {
    let target_and_proj: Vec<&str> = label.split(':').collect();

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
                project.implicit_dependencies.insert(proj.to_string());
                Some(vec![proj.to_string()])
            },
            target: format!("build_{target}"),
        }
    }
}

fn to_attr_deps(deps: Vec<Dependency>) -> hcl::Value {
    let mut labels = vec![];

    for dep in deps {
        let target = dep.target.split("build_").collect::<Vec<&str>>()[1];
        match dep.projects {
            Some(projects) => labels.push(hcl::Value::String(format!("{}:{}", projects[0], target))),
            None => labels.push(hcl::Value::String(format!(":{}", target))),
        }
    }

    hcl::Value::Array(labels)
}

fn from_attr_deps(ctx: &mut TargetContext<'_>, attrs: &HashMap<hcl::Identifier, hcl::Value>) -> Vec<Dependency> {
    attrs
        .get("deps")
        .unwrap_or(&hcl::Value::Array(vec![]))
        .as_array()
        .expect("deps must be an array")
        .into_iter()
        .map(|v| {
            let val = v.as_str().unwrap();
            to_dep(ctx, val)
        })
        .collect()
}

fn generated_file(ctx: &mut TargetContext<'_>, contents: String, outputs: Vec<String>) {
    let path: PathBuf = outputs[0].clone().into();
    //fs::create_dir_all(path.parent().unwrap().to_owned()).unwrap();
    //let mut file = fs::File::create(path.clone()).unwrap();
    //file.write_all(contents.as_bytes()).unwrap();
    let mut proj = ctx.project.borrow_mut();

    proj.implicit_dependencies.insert("tools/mx-build".to_string());
    proj.targets.insert(
        format!("build_{}", ctx.target_name),
        Target {
            inputs: vec![],
            depends_on: vec![Dependency {
                target: String::from("^build"),
                projects: None,
            }],
            outputs: vec![format!("{{workspaceRoot}}/{}", path.display())],
            executor: String::from("./dist/tools/mx-build:generated-file"),
            options: json!({
                "path": path.display().to_string(),
                "contents": contents
            }),
            metadata: None,
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

/// Generates a manifest in the MINI (MeshX INI) format.
///
/// This format maps a source file to its destination in a container with the
/// following line:
///```hcl
///target=source
///```
///
/// The output of this template is a manifest file that lists all packageable
/// elements encountered within `deps`. For more details, see
/// //docs/concepts/build_system/internals/manifest_formats.md
///
/// Parameters
fn generate_mini_manifest(ctx: &mut TargetContext<'_>, depends_on: Vec<Dependency>, mini_manifest_out_dir: String) {
    let main_target = &ctx.target_name;
    let generate_target = format!("{}_dist", ctx.target_name);
    let manifest_file = format!("{}/{}.mini", ctx.target_gen_dir, ctx.target_name);

    ctx.project.borrow_mut().targets.insert(
        format!("build_{}", ctx.target_name),
        Target {
            inputs: vec![Input::DependentTasks {
                dependent_tasks_output_files: "**/*".to_string(),
            }],
            outputs: vec![format!("{{workspaceRoot}}/{manifest_file}")],
            depends_on,
            executor: String::from("./dist/tools/mx-build:mini-manifest"),
            options: json!({
                "outputFileName": manifest_file
            }),
            metadata: None,
        },
    );
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
) -> String {
    let target_name = format!("build_{}", ctx.target_name);
    let mut deps = from_attr_deps(ctx, attrs);

    let package_name = ctx.target_name.to_owned();
    let repository = "meshx.co";

    // LINT.IfChange
    let mini_manifest_out_dir = format!("{}/{}_manifest", ctx.target_out_dir, ctx.target_name);
    let package_out_dir = format!("{}/{}", ctx.target_out_dir, ctx.target_name);
    let package_manifest = format!("{package_out_dir}/package_manifest.json");
    // LINT.ThenChange(//build/packages/exported_fuchsia_package_archive.gni)

    // Generate the "meta/package" file
    let meta_package_target = format!("{}_meta_package", ctx.target_name);

    generate_meta_package(
        &mut ctx.with_target_name(meta_package_target.clone()),
        package_name,
        //forward_variables_from(invoker, ["applicable_licenses", "testonly"]),
        //visibility = [":*"],
        //package_name = package_name,
    );

    // Generate package .mini manifest
    let package_manifest_target = format!("{}_manifest", ctx.target_name);
    generate_mini_manifest(
        &mut ctx.with_target_name(package_manifest_target.clone()),
        deps.clone(),
        mini_manifest_out_dir,
    );

    ctx.project.borrow_mut().targets.insert(
        target_name.clone(),
        Target {
            inputs: vec![],
            outputs: vec![],
            depends_on: vec![
                Dependency {
                    projects: None,
                    target: format!("build_{}", meta_package_target),
                },
                Dependency {
                    projects: None,
                    target: format!("build_{}", package_manifest_target),
                },
            ],
            executor: String::from("nx:noop"),
            options: json!({}),
            metadata: None,
        },
    );

    target_name
}

fn cm(ctx: &mut TargetContext<'_>, attrs: &HashMap<hcl::Identifier, hcl::Value>) -> String {
    let target_name = format!("build_{}", ctx.target_name);
    let deps = from_attr_deps(ctx, attrs);

    let component_name = attrs
        .get("component_name")
        .unwrap_or(&hcl::Value::String(ctx.target_name.to_string()))
        .as_str()
        .expect("component_name must be an string");

    ctx.project.borrow_mut().targets.insert(
        target_name.clone(),
        Target {
            inputs: vec![],
            outputs: vec![],
            depends_on: deps,
            executor: String::from("nx:noop"),
            options: json!({}),
            metadata: None,
        },
    );

    target_name
}

fn process_file_template(sources: Vec<&str>, outputs: &Vec<String>) -> Vec<String> {
    let mut result = Vec::new();

    for source in sources {
        // Extract the source name part by removing the extension
        if let Some((source_name_part, _)) = source.rsplit_once('.') {
            for output in outputs {
                // Replace the {{source_name_part}} placeholder with the actual source name part
                let processed_output = output.replace("{{source_name_part}}", source_name_part);
                result.push(processed_output);
            }
        }
    }

    result
}

/// Declare data files to be accessible at runtime in the distribution.
///
/// A `resource` target looks just like a `copy` target but $outputs are
/// relative target paths.  Using $data_deps to this resource() target in
/// each target whose code uses $outputs at runtime ensures that the files
/// will be present on the system.
///
/// If the `sources` list contains more than one file, the `outputs` should use
/// source expansion template placeholders, such as `{{source_file_part}}`.
///
/// For example:
///
///   //some/dir/BUILD.hcl:
///      resource "testdata" {
///        sources = [
///          "testdata/input.json",
///          "testdata/input_minified.json",
///        ]
///        outputs = [ "data/{{source_file_part}}" ]
///      }
///
/// The above `resource` target maps files in the subdirectory `testdata` to
/// destination paths in a Fuchsia package as follows:
///
///  //some/dir/testdata/input.json           -> data/input.json
///  //some/dir/testdata/input_minified.json  -> data/input_minified.json
///
fn resource(
    ctx: &mut TargetContext<'_>,
    attrs: &HashMap<hcl::Identifier, hcl::Value>,
    root: PathBuf,
    projects: &mut HashMap<String, ProjectConfiguration>,
) -> String {
    let target_name = format!("build_{}", ctx.target_name);
    let deps = from_attr_deps(ctx, attrs);

    let sources: Vec<&str> = attrs
        .get("sources")
        .expect("sources are required")
        .as_array()
        .expect("sources must be an array")
        .into_iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    let outputs: Vec<String> = attrs
        .get("outputs")
        .expect("outputs are required")
        .as_array()
        .expect("outputs must be an array")
        .into_iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect();

    let mut distribution_entries = vec![];
    for source in sources {
        for target in process_file_template(vec![source], &outputs) {
            distribution_entries.push(json!({
                "source": format!("{}/{source}", ctx.root_build_dir),
                "destination": target
                // "label": "_label"
            }))
        }
    }

    ctx.project.borrow_mut().targets.insert(
        target_name.clone(),
        Target {
            inputs: vec![],
            outputs: vec![],
            depends_on: deps,
            executor: String::from("nx:noop"),
            options: json!({}),
            metadata: Some(json!({
                "distribution_entries": distribution_entries
            })),
        },
    );

    target_name
}

fn meshx_component_manifest(
    ctx: &mut TargetContext<'_>,
    attrs: &HashMap<hcl::Identifier, hcl::Value>,
    root: PathBuf,
    projects: &mut HashMap<String, ProjectConfiguration>,
) -> String {
    let mut deps = from_attr_deps(ctx, attrs);

    let component_name_attr = attrs.get("component_name").cloned();
    let component_name = component_name_attr.unwrap_or(hcl::Value::String(ctx.root_target_name.to_owned()));
    let component_name = component_name.as_str().unwrap();

    let manifest_name = format!("{}.cm", component_name);
    let manifest_resource_target = format!("{}_manifest_resource", ctx.target_name);

    deps.append(&mut vec![Dependency {
        projects: None,
        target: format!("build_{}", &manifest_resource_target),
    }]);

    // Process the manifest
    let mut cm_attrs = HashMap::new();
    cm_attrs.insert(hcl::Identifier::sanitized("deps"), to_attr_deps(deps));
    cm_attrs.insert(hcl::Identifier::sanitized("component_name"), hcl::value!("test"));
    let _ = cm(&mut ctx.with_target_name(ctx.target_name.clone()), &cm_attrs);

    // get_target_outputs(":${invoker.target_name}")
    //    [ "meta/$manifest_name" ]
    //    [ ":*" ]

    let mut resource_attrs = HashMap::new();
    resource_attrs.insert(
        hcl::Identifier::sanitized("sources"),
        hcl::value!([format!("meta/{manifest_name}")]),
    );
    resource_attrs.insert(
        hcl::Identifier::sanitized("outputs"),
        hcl::value!([format!("meta/{manifest_name}")]),
    );

    resource(
        &mut ctx.with_target_name(manifest_resource_target.clone()),
        &resource_attrs,
        root,
        projects,
    )
}

/// Defines a MeshX component.
/// See: https://fuchsia.dev/fuchsia-src/development/components/build
///
/// A component is defined by a component manifest.
/// Component manifests typically reference files in the package that they are
/// distributed in. Therefore a component can also have dependencies on
/// `resource()`, such that any package that depends on the component will
/// also include that resource.
///
/// A component is launched by a URL.
/// See: https://fuchsia.dev/fuchsia-src/glossary#component_url
///
/// A component's URL is a function of the name of a package that includes it,
/// and the path within that package to the component's manifest. For instance if
/// you defined the following:
/// ```hcl
/// executable "my_program" {
///   ...
/// }
///
/// meshx_component "my-component" {
///   manifest = "manifest.cml"
///   deps = [ ":my_program" ]
/// }
///
/// meshx_package "my-package" {
///   deps = [ ":my-component" ]
/// }
/// ```
/// The component above will have the following launch URL:
///
///`meshx-pkg://fuchsia.com/my-package#meta/my-component.cm`
///
/// Since the component depends on the executable target, the binary produced by
/// the executable will be packaged with the manifest. Therefore the manifest
/// author can reference the path `bin/my_program`.
///
/// Components may depend on any number of `resource` targets to ensure that
/// any `meshx_package "<target>"` that includes them will include the same resources.
///
/// ```hcl
/// resource "my_file" {
///   sources = [ "my_file.txt" ]
///   outputs = [ "data/{{source_file_part}}" ]
/// }
///
/// meshx_component "my-component" {
///   deps = [ ":my_file" ]
///   ...
/// }
/// ```
fn meshx_component(
    ctx: &mut TargetContext<'_>,
    attrs: &HashMap<hcl::Identifier, hcl::Value>,
    root: PathBuf,
    projects: &mut HashMap<String, ProjectConfiguration>,
) -> String {
    let target_name = format!("build_{}", ctx.target_name);
    let mut deps = from_attr_deps(ctx, attrs);

    // Extract attributes
    let manifest = attrs.get("manifest").map_or(None, |v| v.as_str());
    let cm_label = attrs.get("cm_label").map_or(None, |v| v.as_str());

    let cm_dep = match (manifest, cm_label) {
        (None, Some(cm_label)) => to_dep(ctx, cm_label),
        (Some(manifest), None) => {
            // Compile the manifest from source
            let cm_target = format!("{}_manifest_compile", ctx.target_name);
            let _target = meshx_component_manifest(&mut ctx.with_target_name(cm_target.clone()), attrs, root, projects);
            to_dep(ctx, format!(":{}", cm_target).as_str())
        }
        (Some(_), Some(_)) => panic!("Either manifest or cm_target is needed. Can't be both."),
        (None, None) => panic!("Either manifest or cm_target is needed."),
    };

    deps.append(&mut vec![cm_dep]);

    ctx.project.borrow_mut().targets.insert(
        target_name.clone(),
        Target {
            inputs: vec![],
            outputs: vec![],
            depends_on: deps,
            executor: String::from("nx:noop"),
            options: json!({}),
            metadata: None,
        },
    );

    target_name
}

/// Declares a MIDL library.
///
/// Supported backends: rust, typescript
fn generate_midl(
    ctx: &mut TargetContext<'_>,
    attrs: &HashMap<hcl::Identifier, hcl::Value>,
    root: PathBuf,
    projects: &mut HashMap<String, ProjectConfiguration>,
) -> String {
    let target_name = format!("build_{}", ctx.target_name);
    let deps = from_attr_deps(ctx, attrs);

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

    let inputs: Vec<Input> = sources
        .iter()
        .map(|src| Input::File(format!("{{workspaceRoot}}/{}", src)))
        .collect();

    let mut proj = ctx.project.borrow_mut();
    proj.implicit_dependencies.insert("tools/nx-midl".to_string());
    proj.targets.insert(
        target_name.clone(),
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
            metadata: None,
        },
    );

    target_name
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
                metadata: None,
            },
        );

        let default_project = Rc::new(RefCell::new(ProjectConfiguration {
            name: root.clone().display().to_string(),
            project_type: Some(String::from("library")),
            tags: HashSet::from(["lang:midl".to_string()]),
            targets,
            implicit_dependencies: HashSet::new(),
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
            let root_target_name = target_name.clone();

            let target_out_dir = format!("dist/{}", root.display());
            let target_out_dir = target_out_dir.as_str();

            let target_gen_dir = format!("dist/gen/{}", root.display());
            let target_gen_dir = target_gen_dir.as_str();
            let cwd = env::current_dir().unwrap();

            let mut ctx = TargetContext {
                root_target_name: root_target_name.as_str(),
                target_name,
                target_out_dir,
                target_gen_dir,
                root_build_dir: cwd.to_str().unwrap(),
                project: default_project.clone(),
            };

            let root_target = match block.identifier() {
                "midl" => generate_midl(&mut ctx, &attrs, root.clone(), &mut projects),
                "resource" => resource(&mut ctx, &attrs, root.clone(), &mut projects),
                "meshx_component_manifest" => meshx_component_manifest(&mut ctx, &attrs, root.clone(), &mut projects),
                "meshx_component" => meshx_component(&mut ctx, &attrs, root.clone(), &mut projects),
                "meshx_package" => meshx_package(&mut ctx, &attrs, root.clone(), &mut projects),
                v => panic!("not supported block: {:?}", v),
            };
        }

        let p = (*default_project).borrow();
        let s = p.clone();
        projects.insert(root.clone().display().to_string(), s);

        output.push(json!([input, CreateNodesResult { projects }]));
    }

    serde_json::to_writer(std::io::stdout(), &output)?;

    Ok(())
}
