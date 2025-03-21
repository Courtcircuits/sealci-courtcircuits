## Dumper Architecture

### Overview

Dumper is a VMM that can create and boot quickly virtual machines. It is designed to be lightweight and efficient, making it ideal for use in cloud environments and other resource-constrained settings.

The core components of Dumper include:

- Virtual Machine Manager (VMM)
- Resource Manager
- Network Manager

### Components

Our solution will be build on top of KVM that allow virtualization for linux. It will allow us to quickly boot kernel and create virtual machines.

In order for the machine to be able to perform computation, it needs to have access to the necessary resources. This includes CPU and memory.

In order to configure memory, we will need to allocate memory from the guest machine. 
Then this memory will be passed to KVM in order to manage it. This memory will hold all the data of the virtual machine.
In fact, no disk will be allocated to the virtual machine and it will only perform write and read operations in memory.

Of course, with memory comes compute. In order to perform computation, we will need to allocate CPU resources. 
[COMPLETE CPU PART]


As the virtual machine will be used in a CI/CD pipeline, we will need to ensure that we can execute commands inside the virtual machine.
We will allocate an standard input/output device to the virtual machine through a serial port.

Some of the commands could need the virtual machine to be able to perform network calls. In order to do so, we will need to allocate network resources from virtio device.
Virtio allocate some of the memory virtual machine to read and write packets that will be transferred to the guest network interface.

### Crate use && goals

The crate must provide a way to boot and destroy quickly virtual machines.

We will provide an example on what should be allocated to the virtual machine  in order to perform CI/CD operations. 
