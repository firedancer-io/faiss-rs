fn main() {
    #[cfg(feature = "static")]
    static_link_faiss();
    #[cfg(not(feature = "static"))]
    println!("cargo:rustc-link-lib=faiss_c");
}

#[cfg(feature = "static")]
fn static_link_faiss() {
    let mut cfg = cmake::Config::new("faiss");
    cfg.define("FAISS_ENABLE_C_API", "ON")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("FAISS_ENABLE_GPU", if cfg!(feature = "gpu") {
            "ON"
        } else {
            "OFF"
        })
        .define("FAISS_ENABLE_PYTHON", "OFF")
        .define("BUILD_TESTING", "OFF")
        .very_verbose(true);
    let dst = cfg.build();

    // cmake installs to lib/ on most systems, but lib64/ on RHEL/EL9.
    // Add both to the search path so linking works on either layout.
    let faiss_lib = dst.join("lib");
    let faiss_lib64 = dst.join("lib64");
    let faiss_c_location = dst.join("build/c_api");
    if faiss_lib.is_dir() {
        println!(
            "cargo:rustc-link-search=native={}",
            faiss_lib.display()
        );
    }
    if faiss_lib64.is_dir() {
        println!(
            "cargo:rustc-link-search=native={}",
            faiss_lib64.display()
        );
    }
    println!(
        "cargo:rustc-link-search=native={}",
        faiss_c_location.display()
    );
    println!("cargo:rustc-link-lib=static=faiss_c");
    println!("cargo:rustc-link-lib=static=faiss");
    link_cxx();
    println!("cargo:rustc-link-lib=gomp");
    link_blas();
    if cfg!(feature = "gpu") {
        let cuda_path = cuda_lib_path();
        println!("cargo:rustc-link-search=native={}/lib64", cuda_path);
        println!("cargo:rustc-link-lib=cudart");
        println!("cargo:rustc-link-lib=cublas");
    }
}

#[cfg(feature = "static")]
fn link_blas() {
    // Try blas+lapack first; fall back to openblas (common on RHEL/EL9
    // where only libopenblas.so is installed, not libblas.so/liblapack.so).
    let has_blas = std::process::Command::new("sh")
        .args(["-c", "ldconfig -p 2>/dev/null | grep -q 'libblas\\.so'"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if has_blas {
        println!("cargo:rustc-link-lib=blas");
        println!("cargo:rustc-link-lib=lapack");
    } else {
        println!("cargo:rustc-link-lib=openblas");
    }
}

#[cfg(feature = "static")]
fn link_cxx() {
    let cxx = match std::env::var("CXXSTDLIB") {
        Ok(s) if s.is_empty() => None,
        Ok(s) => Some(s),
        Err(_) => {
            let target = std::env::var("TARGET").unwrap();
            if target.contains("msvc") {
                None
            } else if target.contains("apple")
                | target.contains("freebsd")
                | target.contains("openbsd")
            {
                Some("c++".to_string())
            } else {
                Some("stdc++".to_string())
            }
        }
    };
    if let Some(cxx) = cxx {
        println!("cargo:rustc-link-lib={}", cxx);
    }
}

#[cfg(feature = "static")]
fn cuda_lib_path() -> String {
    // look for CUDA_PATH in environment,
    // then CUDA_LIB_PATH,
    // then CUDA_INCLUDE_PATH
    if let Ok(cuda_path) = std::env::var("CUDA_PATH") {
        return cuda_path;
    }
    if let Ok(cuda_lib_path) = std::env::var("CUDA_LIB_PATH") {
        return cuda_lib_path;
    }
    if let Ok(cuda_include_path) = std::env::var("CUDA_INCLUDE_PATH") {
        return cuda_include_path;
    }

    panic!("Could not find CUDA: environment variables `CUDA_PATH`, `CUDA_LIB_PATH`, or `CUDA_INCLUDE_PATH` must be set");
}
