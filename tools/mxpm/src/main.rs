mod cmd;

use clap::{crate_authors, crate_version};
use clap::{App, Arg, SubCommand};
use directories::ProjectDirs;
use futures_util::future;
use http::response::Builder as ResponseBuilder;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, StatusCode};
use hyper::{Body, Request, Response, Server};
use hyper_staticfile::Static;
use std::fs;
use std::io::Error as IoError;
use std::net::SocketAddr;
use std::path::Path;

fn publish_package(_repo_path: &Path) {}

async fn handle_package_request<B>(req: Request<B>, static_: Static) -> Result<Response<Body>, IoError> {
    if req.uri().path() == "/" {
        let res = ResponseBuilder::new()
            .status(StatusCode::MOVED_PERMANENTLY)
            .header(header::LOCATION, "/hyper_staticfile/")
            .body(Body::empty())
            .expect("unable to build response");
        Ok(res)
    } else {
        static_.clone().serve(req).await
    }
}

async fn start_repository_server(addr_str: &str, repo_path: &Path) {
    let addr: SocketAddr = addr_str.parse().unwrap();
    let static_ = hyper_staticfile::Static::new(repo_path.to_str().unwrap());

    let make_service = make_service_fn(|_| {
        let static_ = static_.clone();
        future::ok::<_, hyper::Error>(service_fn(move |req| handle_package_request(req, static_.clone())))
    });

    let server = Server::bind(&addr).serve(make_service);

    println!("Serving packages on http://{}", addr);
    println!("Local repo path: {:?}", repo_path);

    server.await.unwrap();
}

fn main() {
    let proj_dirs = ProjectDirs::from("", "MeshX", "PackageManager").unwrap();
    let local_repo_path = proj_dirs.config_dir();

    fs::create_dir_all(local_repo_path).unwrap();

    let publish_cmd = SubCommand::with_name("publish")
        .version(crate_version!())
        .about("publish a package to a local repository");

    let dev_cmd = SubCommand::with_name("dev")
        .about("TODO")
        .args(&[Arg::with_name("input_dir")
            .short("d")
            .long("dir")
            .default_value("./")]);

    let serve_cmd = SubCommand::with_name("serve")
        .version(crate_version!())
        .args(&[
            Arg::with_name("host").short("h").long("host").default_value("0.0.0.0"),
            Arg::with_name("port").short("p").long("port").default_value("9087"),
        ])
        .about("serve a local repository");

    let pack_cmd = SubCommand::with_name("pack")
        .version(crate_version!())
        .args(&[
            Arg::with_name("package directory")
                .short("d")
                .long("dir")
                .default_value("./"),
            Arg::with_name("output dir")
                .short("o")
                .long("out")
                .default_value("./out"),
        ])
        .about("pack the package into a single .tar");

    let matches = App::new("MeshX Package Manager")
        .bin_name("meshx")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Package Manager for install, upload, build packages in MeshX Workspaces")
        .subcommands([dev_cmd, publish_cmd, serve_cmd, pack_cmd])
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("pack") {
        let input_dir = matches.value_of("package directory").unwrap();
        let output_dir = matches.value_of("output dir").unwrap();
        cmd::pack::archive_package(input_dir, output_dir)
    }

    if let Some(_matches) = matches.subcommand_matches("publish") {
        publish_package(local_repo_path)
    }

    if let Some(_matches) = matches.subcommand_matches("serve") {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");

        // Combine it with a `LocalSet,  which means it can spawn !Send futures...
        let local = tokio::task::LocalSet::new();
        local.block_on(&rt, async move {
            start_repository_server("127.0.0.1:1337", local_repo_path).await;
        });
    }

    if let Some(arg_matches) = matches.subcommand_matches("dev") {
        let indir = arg_matches.value_of("input_dir").unwrap();

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");

        // Combine it with a `LocalSet,  which means it can spawn !Send futures...
        let local = tokio::task::LocalSet::new();
        local.block_on(&rt, async move {
            cmd::serve::start_dev_server(indir, "127.0.0.1:8081").await;
        });
    }
}
