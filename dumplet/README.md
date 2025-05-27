# **Dumplet 🐳 → 📁**
🔹 **Dumplet** is a Rust **crate** and **CLI tool** that extracts the filesystem of a **Docker image** and saves it as a `.tar` archive.  
🔹 It uses the **Bollard** crate to interact with the Docker API.

---

## **✨ Features**
✅ Pulls a **Docker image** (always forced for now).  
✅ Creates a **temporary container** _(without running it)_.  
✅ Exports the **container’s filesystem** as a `.tar` archive.  
✅ **Automatically removes** the container after extraction.  
✅ Can be used as a **Rust library** or as a **CLI**.

---

## **📦 Installation**
### **Prerequisites**
- **Docker** must be installed and running on your system.
    - Install Docker: [https://docs.docker.com/get-docker/](https://docs.docker.com/get-docker/)
- Rust & Cargo installed: [https://rustup.rs/](https://rustup.rs/)

### **As a CLI Tool (Recommended if you just want to export images)**
1. Clone the repository:
    ```sh
    git clone https://github.com/dev-sys-do/sealci
    cd sealci/dumplet
    ```
2. Build the binary:
    ```sh
    cargo build --release
    ```
3. Use it:
    ```sh
    ./target/release/dumplet <IMAGE_NAME> <OUTPUT_PATH>
    ```
   **Example:**
    ```sh
    ./target/release/dumplet ubuntu:latest ./tmp/ubuntu_fs.tar
    ```

### **As a Rust Library**
If you want to use Dumplet’s logic in your own Rust project, add it to your `Cargo.toml`:
```toml
[dependencies]
dumplet = { path = "../dumplet" }
```

Example usage in your Rust code:
```rust
use dumplet::export_docker_image;

#[tokio::main]
async fn main() {
    export_docker_image("ubuntu:latest", "/tmp/ubuntu_fs.tar").await.unwrap();
}
```

---

## **📂 Output Example**
After running the above example, you’ll get:
```sh
/tmp/ubuntu_fs.tar
```
You can extract it with:
```sh
tar -xf /tmp/ubuntu_fs.tar -C /your/destination
```

---

## **👨‍💻 Contributing**
Want to contribute? Feel free to **open an issue or a pull request**!  
To run the project locally:
```sh
git clone https://github.com/dev-sys-do/sealci
cd sealci/dumplet
cargo build
cargo test
```

---

🔥 **Ready to use Dumplet?** Pull your first image now! 🚀  
If you have any questions, feel free to open an **issue** or contribute! 😊  
