use std::env;
use std::path::PathBuf;

#[cfg(windows)]
fn build_windows() {
    let file = "src/platform/windows.cc";
    let file2 = "src/platform/windows_delete_test_cert.cc";

    // 编译 Windows 平台相关的 C++ 源文件
    cc::Build::new().file(file).file(file2).compile("windows");

    // 输出警告信息，确认进入了 `build_windows` 函数
    println!("cargo:warning=Entering build_windows function.");

    // 链接 Windows Media Foundation 所需的库
    println!("cargo:rustc-link-lib=mfplat");
    println!("cargo:rustc-link-lib=mf");
    println!("cargo:rustc-link-lib=mfuuid");
    println!("cargo:rustc-link-lib=strmiids");// ICode

    // 指定 Windows SDK 中 `strmiids.lib` 的路径
    let sdk_path = "C:\\Program Files (x86)\\Windows Kits\\10\\Lib\\10.0.22621.0\\um\\x64";
    println!("cargo:warning=Adding link search path: {}", sdk_path);
    println!("cargo:rustc-link-search=native={}", sdk_path);

    // 链接 Windows API 库
    println!("cargo:rustc-link-lib=WtsApi32");

    // 设置文件的重新编译检查
    println!("cargo:rerun-if-changed={}", file);
    println!("cargo:rerun-if-changed={}", file2);

    // 添加 vcpkg 的库搜索路径
    if let Ok(vcpkg_root) = env::var("VCPKG_ROOT") {
        let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "x64".to_owned());
        let lib_path = PathBuf::from(format!(
            "{}\\installed\\{}-windows-static\\lib",
            vcpkg_root, target_arch
        ));

        // 输出警告信息，查看 vcpkg 库路径
        println!("cargo:warning=Trying to link with vcpkg libraries in path: {}", lib_path.display());

        // 指定 vcpkg 库搜索路径
        println!("cargo:rustc-link-search=native={}", lib_path.display());

        // 添加需要链接的库
        let libraries = [
            "avcodec",   // FFmpeg
            "avformat",  // FFmpeg
            "avutil",    // FFmpeg
            "swresample",// FFmpeg
            "mfx",       // Intel Media SDK (mfx-dispatch)
            "vpx",       // libvpx
            "yuv",       // libyuv
            "opus",      // Opus codec
            "aom",       // AOMedia Video 1 (AV1)
        ];

        for lib in libraries.iter() {
            // 输出警告信息，显示将要链接的库名称
            println!("cargo:warning=Linking with library: {}", lib);
            println!("cargo:rustc-link-lib=static={}", lib);
        }
    } else {
        eprintln!("VCPKG_ROOT is not set. Please set the VCPKG_ROOT environment variable.");
    }
}

#[cfg(target_os = "macos")]
fn build_mac() {
    let file = "src/platform/macos.mm";
    let mut b = cc::Build::new();

    // 检查 macOS 版本，如果是 10.14，则添加编译标志
    if let Ok(os_version::OsVersion::MacOS(v)) = os_version::detect() {
        let v = v.version;
        if v.contains("10.14") {
            b.flag("-DNO_InputMonitoringAuthStatus=1");
        }
    }
    b.file(file).compile("macos");

    // 设置重新编译检查
    println!("cargo:rerun-if-changed={}", file);
}

#[cfg(all(windows, feature = "inline"))]
fn build_manifest() {
    use std::io::Write;
    if std::env::var("PROFILE").unwrap() == "release" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("res/icon.ico")
            .set_language(winapi::um::winnt::MAKELANGID(
                winapi::um::winnt::LANG_ENGLISH,
                winapi::um::winnt::SUBLANG_ENGLISH_US,
            ))
            .set_manifest_file("res/manifest.xml");

        match res.compile() {
            Err(e) => {
                write!(std::io::stderr(), "{}", e).unwrap();
                std::process::exit(1);
            }
            Ok(_) => {}
        }
    }
}

fn install_android_deps() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os != "android" {
        return;
    }

    let mut target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_arch == "x86_64" {
        target_arch = "x64".to_owned();
    } else if target_arch == "x86" {
        target_arch = "x86".to_owned();
    } else if target_arch == "aarch64" {
        target_arch = "arm64".to_owned();
    } else {
        target_arch = "arm".to_owned();
    }

    let target = format!("{}-android", target_arch);
    let vcpkg_root = env::var("VCPKG_ROOT").unwrap();
    let mut path: PathBuf = vcpkg_root.into();
    path.push("installed");
    path.push(target);

    println!(
        "cargo:rustc-link-search=native={}",
        path.join("lib").to_str().unwrap()
    );
    println!("cargo:rustc-link-lib=ndk_compat");
    println!("cargo:rustc-link-lib=oboe");
    println!("cargo:rustc-link-lib=oboe_wrapper");
    println!("cargo:rustc-link-lib=c++");
    println!("cargo:rustc-link-lib=OpenSLES");
}

fn main() {
    hbb_common::gen_version();
    install_android_deps();

    #[cfg(all(windows, feature = "inline"))]
    build_manifest();

    #[cfg(windows)]
    build_windows();

    #[cfg(target_os = "macos")]
    build_mac();

    println!("cargo:warning=Finished running build script.");
    println!("cargo:rerun-if-changed=build.rs");
}
