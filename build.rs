fn main() {
    let cores = num_cpus::get();
    let tpcl2 = (cores as f64).log2().floor() as u32;

    // the num cores is read in through env for the rust build too.
    println!("cargo:rustc-env=TPC={}", cores);

    println!("cargo:rerun-if-changed=src/hvm.c");
    println!("cargo:rerun-if-changed=src/hvm.cu");

    match cc::Build::new()
        .file("src/hvm.c")
        .opt_level(3)
        .warnings(false)
        .define("TPC_L2", &*tpcl2.to_string())
        .try_compile("hvm-c")
    {
        Ok(_) => println!("cargo:rustc-cfg=feature=\"c\""),
        Err(e) => {
            println!(
                "cargo:warning=\x1b[1m\x1b[31mWARNING: Failed to compile hvm.c:\x1b[0m {}",
                e
            );
            println!("cargo:warning=Ignoring hvm.c and proceeding with build. \x1b[1mThe C runtime will not be available.\x1b[0m");
        }
    }

    // Builds hvm.cu
    if std::process::Command::new("nvcc")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
    {
        if let Ok(cuda_path) = std::env::var("CUDA_HOME") {
            println!("cargo:rustc-link-search=native={}/lib64", cuda_path);
        } else {
            println!("cargo:rustc-link-search=native=/usr/local/cuda/lib64");
        }

        cc::Build::new()
            .cuda(true)
            .file("src/hvm.cu")
            .flag("-diag-suppress=177") // variable was declared but never referenced
            .flag("-diag-suppress=550") // variable was set but never used
            .flag("-diag-suppress=20039") // a __host__ function redeclared with __device__, hence treated as a __host__ __device__ function
            .compile("hvm-cu");

        println!("cargo:rustc-cfg=feature=\"cuda\"");
    } else {
        println!("cargo:warning=\x1b[1m\x1b[31mWARNING: CUDA compiler not found.\x1b[0m \x1b[1mHVM will not be able to run on GPU.\x1b[0m");
    }
}
