# **Dumplet 🐳 → 📁**
🔹 **Dumplet** is a Rust crate that extracts the filesystem of a **Docker image** and saves it as a `.tar` archive.  
🔹 It uses the **Bollard** crate to interact with the Docker API.

---

## **✨ Features**
✅ Pulls a **Docker image** (always forced for now).  
✅ Creates a **temporary container** _(without running it)_.  
✅ Exports the **container’s filesystem** as a `.tar` archive.  
✅ **Automatically removes** the container after extraction.

---

## **📦 Installation**
### **Prerequisites**
- **Docker** must be installed and running on your system.
    - Install Docker: [https://docs.docker.com/get-docker/](https://docs.docker.com/get-docker/)
- Rust & Cargo installed: [https://rustup.rs/](https://rustup.rs/)

### **Add Dumplet to your Rust project**
Dumplet is a local crate for now. Add it to your `Cargo.toml`:
```toml
[dependencies]
dumplet = { path = "../dumplet" }
``` 

---

## **🚀 Usage**
### **Basic Example**
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
git clone https://github.com/yourusername/dumplet.git
cd dumplet
cargo build
cargo test
```

---

🔥 **Ready to use Dumplet?** Pull your first image now! 🚀  
If you have any questions, feel free to open an **issue** or contribute! 😊  
