use std::{
    env::current_dir,
    fs::{copy, create_dir, create_dir_all, read_to_string, File},
    io::Write,
    path::PathBuf,
    process::{self, exit},
};

use clap::Parser;
use toml::Table;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Arguments {
    /// Whether to build the driver for release or for debugging
    #[arg(short, long)]
    release: bool,

    /// Whether to install the resulting driver to /Library/Audio/Plug-Ins/HAL (note that this requires running as root)
    #[arg(short, long)]
    install: bool,

    /// Path to the driver cargo package (cwd by default)
    #[arg(short, long)]
    package: Option<PathBuf>,
}

fn main() {
    let args = Arguments::parse();
    let package_dir = match args.package {
        Some(path) => path,
        None => current_dir().unwrap(),
    };
    let Ok(toml) = read_to_string(package_dir.join("Cargo.toml")) else {
        eprintln!("Error: Could not read Cargo.toml in the specified directory. Please ensure it is a valid Cargo package");
        exit(-1)
    };
    let Ok(t) = toml.parse::<Table>() else {
        eprintln!("Error: Cargo.toml file for this package is malformed");
        exit(-1)
    };
    let Some(Some(Some(package_name))) =
        t.get("package").map(|v| v.get("name").map(|v| v.as_str()))
    else {
        eprintln!("Error: Cargo.toml file does not contain a package name!");
        exit(-1)
    };
    let Some(Some(Some(package_version))) = t
        .get("package")
        .map(|v| v.get("version").map(|v| v.as_str()))
    else {
        eprintln!("Error: Cargo.toml file does not contain a package version!");
        exit(-1)
    };
    if t.get("lib").is_none() {
        eprintln!("Error: Package is not a library");
        exit(-1)
    }
    let Some(Some(Some(Some(lib_type)))) = t["lib"]
        .get("crate-type")
        .map(|a| a.as_array().map(|v| v.first().map(|v| v.as_str())))
    else {
        eprintln!("Error: Package lib type is not specified! Please specify it to be \"cdylib\".");
        exit(-1)
    };
    if lib_type != "cdylib" {
        eprintln!("Error: Package lib type is specified but not \"cdylib\". Please change it to \"cdylib\".");
        exit(-1)
    };
    let mut proc = process::Command::new("cargo");
    proc.arg("build").current_dir(&package_dir);
    if args.release {
        proc.arg("-r");
    }
    let mut c = proc.spawn().expect("Error: failed to start cargo");
    c.wait().expect("Error: Cargo wasn't running");

    let target_dir = if std::fs::read_dir(&package_dir.join("target")).is_ok() {
        package_dir.join("target")
    } else {
        let t = std::fs::canonicalize(&package_dir)
            .unwrap()
            .parent()
            .unwrap()
            .join("target");
        if std::fs::read_dir(&t).is_ok() {
            t
        } else {
            eprintln!("Error: could not determine the \"target\" folder for this package");
            exit(-1)
        }
    };
    let outpath = target_dir.join(match args.release {
        true => "release",
        false => "debug",
    });
    let libpath = outpath.join(format!("lib{}.dylib", package_name));
    if File::open(&libpath).is_err() {
        eprintln!("Error: package binary does not exist or cannot be accessed");
        exit(-1)
    }
    let drvpath = outpath.join(format!("{}.driver", package_name));

    let _ = std::fs::remove_dir_all(&drvpath);

    create_dir_all(drvpath.join("Contents").join("MacOS")).unwrap();
    File::create(drvpath.join("Contents").join("Info.plist"))
        .unwrap()
        .write_all(&make_info_plist(package_name, package_version).as_bytes())
        .unwrap();
    copy(
        libpath,
        drvpath.join("Contents").join("MacOS").join(package_name),
    )
    .unwrap();
    if args.install {
        let installpath = format!("/library/Audio/Plug-Ins/HAL/{}.driver", package_name);
        let _ = std::fs::remove_dir_all(&installpath);
        copy_dir(&drvpath, &installpath).unwrap();
        println!("installed driver to {}", &installpath);
    }
    println!("Built driver to {:?}", &drvpath);
}

fn make_info_plist(package_name: &str, package_version: &str) -> String {
    let uuid = Uuid::new_v3(&Uuid::NAMESPACE_URL, &package_name.as_bytes());
    format!(
        r##"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>English</string>
	<key>CFBundleExecutable</key>
	<string>{}</string>
	<key>CFBundleIdentifier</key>
	<string>com.rustaudio.{}</string>
	<key>CFBundleInfoDictionaryVersion</key>
	<string>6.0</string>
	<key>CFBundleName</key>
	<string>{}</string>
	<key>CFBundlePackageType</key>
	<string>BNDL</string>
	<key>CFBundleShortVersionString</key>
	<string>{}</string>
	<key>CFBundleSignature</key>
	<string>????</string>
	<key>CFBundleSupportedPlatforms</key>
	<array>
		<string>MacOSX</string>
	</array>
	<key>CFBundleVersion</key>
	<string>1</string>
	<key>CFPlugInFactories</key>
	<dict>
		<key>{}</key>
		<string>create_driver</string>
	</dict>
	<key>CFPlugInTypes</key>
	<dict>
		<key>443ABAB8-E7B3-491A-B985-BEB9187030DB</key>
		<array>
			<string>{}</string>
		</array>
	</dict>
</dict>
</plist>"##,
        package_name, package_name, package_name, package_version, &uuid, &uuid
    )
}

//Thanks StackOverflow :)
use std::fs;
use std::path::Path;

pub fn copy_dir<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), std::io::Error> {
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            fs::create_dir_all(&dest)?;
        }

        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        fs::copy(&path, &dest_path)?;
                    }
                    None => {}
                }
            }
        }
    }

    Ok(())
}
