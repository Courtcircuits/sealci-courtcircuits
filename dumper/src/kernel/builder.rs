use std::{error::Error, path::PathBuf};

pub struct KernelBuilder {
    init_ram_fs: PathBuf,
    build_dir: PathBuf,
}

impl KernelBuilder {
    pub fn new(init_ram_fs: PathBuf, build_dir: PathBuf) -> Self {
        KernelBuilder {
            init_ram_fs,
            build_dir,
        }
    }
    
    pub fn build(self) -> Result<PathBuf, Box<dyn Error>> {
        let build_dir = PathBuf::from(self.build_dir);
        let kernel_image = build_dir.join("vmlinux");

        // Build the kernel in a directory of your choice
        let build_command = format!(
            "KCFLAGS=\"-Wa,-mx86-used-note=no\" make O={} bzImage -j `nproc`",
            build_dir.display()
        );
        std::process::Command::new("sh")
            .arg("-c")
            .arg(build_command)
            .output()
            .map_err(|e| format!("Failed to build kernel: {}", e))?;
        Ok(kernel_image)
    }
}
