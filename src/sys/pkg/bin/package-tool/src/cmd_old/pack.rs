use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tar::Builder;
use super::models::PackageMetadata;

pub fn archive_package(input_dir: &str, output_dir: &str) {
    let input_dir = Path::new(input_dir);
    let manifest_path = input_dir.join("meta.yml");
  
    let manifest: PackageMetadata = {
        let s = std::fs::read_to_string(manifest_path.clone()).unwrap();
        serde_yaml::from_str(s.as_str()).unwrap()
    };

    let out_path: &Path = Path::new(output_dir);

    std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();

    let file = File::create(output_dir).unwrap();
    let mut builder = Builder::new(file);

    // Append resource files
    //for manifest in &manifest.manifests {
    ////    let manifest_file_name = Path::new(manifest).file_name().unwrap();
    //    let manifest_local_path = Path::new("manifests").join(manifest_file_name);
    //
    //    let host_path = input_dir.join(manifest);
    //    builder.append_path_with_name(host_path, manifest_local_path).unwrap();
    //}

    // Append component sources

    for source in &manifest.files {
        let host_path = input_dir.join(source);
        println!("{:?}", host_path);

        builder.append_path_with_name(host_path, Path::new(source)).unwrap();
    }

    builder.append_path_with_name(manifest_path, "meta.yml").unwrap();

    builder.finish().unwrap();

    let file = File::open(out_path.clone()).unwrap();
    let mut file = BufReader::new(file);

    let s = adler::adler32(&mut file).unwrap();

    println!("Package created, {:?}, sum = {}", out_path, s);

    //builder.append_dir_all("web", web_path).unwrap();
    //builder.append_dir_all("resources", resources_path).unwrap();
}