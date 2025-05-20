# Unattended KVM Install

Automatic AlmaLinux/Rocky Linux Install# Unattended KVM Install

`unattended-kvm-install` is a command-line tool designed to automate the installation of KVM virtual machines, with a focus on AlmaLinux and Rocky Linux distributions. It simplifies the process by generating kickstart files, creating VMs using `virt-install`, and allowing for batch installations from a CSV file.

## Features

*   **Kickstart Generation:** Automatically generates kickstart files based on provided options.
*   **VM Creation:** Creates KVM virtual machines using `virt-install` with the generated kickstart file.
*   **Password Encryption:** Provides a utility to encrypt passwords for use in kickstart files.
*   **Batch Installation:** Supports installing multiple VMs with varying configurations defined in a CSV file.
*   **Flexible Configuration:** Allows a combination of global and VM-specific options for batch installs.

## Prerequisites

*   Rust and Cargo (for building)
*   KVM installed and configured on your system.
*   `virt-install` command-line tool.

## Building

To build the project, clone the repository and run:

```sh
cargo build
```

The executable will be located in `target/debug/unattended-kvm-install`. For a release build, use `cargo build --release`.

## Usage

The tool uses subcommands to perform different actions.

```sh
unattended-kvm-install <COMMAND>
```

### Commands

#### 1. `encrypt-passwd`

Encrypts a password for use in kickstart files. It will prompt for the password and its confirmation.

```sh
unattended-kvm-install encrypt-passwd
```

#### 2. `kickstart`

Generates a kickstart file based on the provided options and prints it to standard output.

**Example:**
```sh
unattended-kvm-install kickstart --vm-name my-vm --text --rootpw-locked --root-sshkey "your-ssh-public-key" --username testuser --user-plain "password123" > my-vm.ks
```
This command will generate a kickstart configuration. You can see all available options with `unattended-kvm-install kickstart --help`.

#### 3. `create-vm`

Creates a VM using `virt-install`. It can optionally take a pre-existing kickstart file.

**Example (without kickstart, manual install):**
```sh
unattended-kvm-install create-vm --vm-name my-vm --iso /path/to/almalinux.iso
```

**Example (with a kickstart file):**
```sh
unattended-kvm-install create-vm --vm-name my-vm --iso /path/to/almalinux.iso --kickstart ./my-vm.ks
```
You can see all available options with `unattended-kvm-install create-vm --help`.

#### 4. `run-all`

Combines kickstart generation and VM creation into a single step. It generates a temporary kickstart file and uses it to create the VM.

**Example:**
```sh
unattended-kvm-install run-all \
  --vm-name my-vm1 \
  --iso /path/to/AlmaLinux-8.8-x86_64-dvd.iso \
  --disk-size 50 \
  --memory 2048 \
  --vcpu 2 \
  --network bridge=br0 \
  --text \
  --timezone Asia/Tokyo \
  --keyboard jp \
  --language ja_JP.UTF-8 \
  --packages @standard --packages zsh \
  --rootpw-locked \
  --root-sshkey="ssh-rsa AAAA..." \
  --username=myuser \
  --user-plain="securepassword" \
  --user-sshkey="ssh-rsa BBBB..." \
  --user-groups=wheel
```
This command will first generate a kickstart file with the specified parameters and then immediately use it to create a VM named `my-vm1`.
You can see all available options with `unattended-kvm-install run-all --help`.

#### 5. `batch-install`

Installs multiple VMs based on configurations from a global options file and a CSV file.

*   **Global Options File:** A text file where each line is a command-line argument (e.g., `testdata/global.txt`). These options apply to all VMs in the batch.
*   **CSV Options File:** A CSV file where the first row defines option names (without `--`) and subsequent rows define values for each VM (e.g., `testdata/list.csv`). These options are specific to each VM and override or supplement global options.

**Example:**

Given `testdata/global.txt`:
```
--network=bridge=br0
--iso=/nfs1/data/linux-iso/AlmaLinux-8.8-x86_64-dvd.iso
--memory=8192
--text
--timezone=Asia/Tokyo
--keyboard=jp
--language=ja_JP.UTF-8
--rootpw-locked
--root-sshkey=ssh-rsa AAAAB3N...
--username=test
--user-crypt=$6$cTgeUX...
--user-sshkey=ssh-rsa AAAAB3N...
--user-uid=5000
--user-gid=5000
--user-groups=wheel
```

And `testdata/list.csv`:
```csv
vm-name,disk-size
vm1,70
vm2,100
vm3,60
```

Run batch install:
```sh
unattended-kvm-install batch-install --global-options testdata/global.txt --csv-options testdata/list.csv
```
This will create `vm2` with a disk size of 100GB, using other options from `global.txt`. Lines in the CSV starting with `#` are ignored.
You can see all available options with `unattended-kvm-install batch-install --help`.

## Configuration Details

*   Most options for kickstart generation and VM creation are exposed as command-line flags. Use `--help` on subcommands (e.g., `unattended-kvm-install run-all --help`) to see all available options.
*   For `batch-install`, the options from the global file and the CSV file are combined. If an option is present in both, the CSV value typically takes precedence for that specific VM.
*   Boolean flags in CSV: Use `TRUE` for enabling a flag (e.g., `--text` becomes a column `text` with value `TRUE`) and `FALSE` to explicitly not include the flag (though omitting it usually has the same effect if the flag isn't a default).

## How it Works

1.  **Kickstart Generation (`kickstart::Kickstart`)**:
    *   Takes various parameters (network settings, system settings, user passwords, SSH keys, storage layout) as input.
    *   Constructs a complete kickstart file string.
    *   Handles password input securely if interactive password setting is chosen.

2.  **VM Creation (`createvm::CreateVmBase`)**:
    *   Constructs a `virt-install` command with parameters like VM name, disk size, memory, vCPUs, ISO location, and network settings.
    *   If a kickstart file path is provided, it injects it into the `virt-install` command using `--initrd-inject` and appropriate `--extra-args`.
    *   Executes the `virt-install` command (usually with `sudo`).

3.  **Run All (`runall::RunAll`)**:
    *   Uses `kickstart::Kickstart::generate()` to create the kickstart content.
    *   Writes this content to a temporary file.
    *   Calls `createvm::CreateVmBase::create_vm()` with the path to the temporary kickstart file.

4.  **Batch Install (`batch_install::BatchInstall`)**:
    *   Reads global options from the specified text file.
    *   Reads VM-specific options from the CSV file using `options_from_csv::generate_options_from_csv()`.
    *   For each VM entry in the CSV:
        *   Combines global and VM-specific options.
        *   Parses these combined options as if they were command-line arguments for the `run-all` command.
        *   Executes the `run-all` logic for that VM.

5.  **Password Handling (`passwd::Passwd`)**:
    *   Provides functions to read passwords securely from the terminal.
    *   Uses `sha-crypt` crate to generate SHA512-crypted password strings suitable for kickstart files.

6.  **CSV Parsing (`options_from_csv::generate_options_from_csv`)**:
    *   Reads a CSV file.
    *   Interprets the header row as option names.
    *   For each data row, constructs a list of command-line arguments (e.g., `column_name` with value `xyz` becomes `"--column_name", "xyz"`).
    *   Handles `TRUE` values by just adding the flag (e.g., `"--flag_name"`) and `FALSE` values by omitting the flag.
