use std::{error::Error, path::PathBuf};

pub struct KernelBuilder {
    /// The file system that will be used to boot the kernel.
    init_ram_fs: PathBuf,
    /// The configuration file name for the kernel.
    /// It must be in the same directory as the kernel source code.
    config: String,
    /// The directory where the kernel will be built.
    build_dir: PathBuf,
}

impl KernelBuilder {
    pub fn new(init_ram_fs: PathBuf, build_dir: PathBuf, config: String) -> Self {
        KernelBuilder {
            init_ram_fs,
            config,
            build_dir,
        }
    }
    
    
    pub fn build(self) -> Result<PathBuf, Box<dyn Error>> {
        let build_dir = PathBuf::from(self.build_dir);
        let kernel_image = build_dir.join("vmlinux");
        let build_dir_parsed = build_dir.to_str().ok_or("Invalid build directory")?;

        // Build the kernel in a directory of your choice
        let build_command = format!("make vmlinux -j `nproc`");

        std::process::Command::new("sh")
            .env("KCFLAGS", "-Wa,-mx86-used-note=no")
            .env("KCONFIG_CONFIG", self.config)
            .env("KBUILD_OUTPUT", build_dir_parsed)
            .arg("-c")
            .arg(build_command)
            .output()
            .map_err(|e| format!("Failed to build kernel: {}", e))?;
        Ok(kernel_image)
    }
}
