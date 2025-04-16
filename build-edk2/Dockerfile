FROM docker.io/debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    g++ \
    gcc \
    gcc-aarch64-linux-gnu \
    git \
    make \
    mono-complete \
    python3 \
    python3-venv \
    uuid-dev \
    wget

# Use the 2023.09.13 nightly GCC 5 toolchain for RISCV64.
RUN wget -q -O gcc-riscv.tar.gz https://github.com/riscv-collab/riscv-gnu-toolchain/releases/download/2023.09.13/riscv64-elf-ubuntu-22.04-gcc-nightly-2023.09.13-nightly.tar.gz
RUN echo "c3b849d25a43d74e7eb1cf739a8a9fadba5f997c4814732650f36ae250a8ab00 gcc-riscv.tar.gz" | sha256sum --check
RUN tar -C /opt -xzf gcc-riscv.tar.gz

# Use git rather than a release tarball, since the tarball is missing
# submodules.
ARG git_tag
RUN git clone --branch "${git_tag}" https://github.com/tianocore/edk2.git
WORKDIR /edk2
RUN git submodule update --init

# Set up deps.
RUN python3 -m venv venv
RUN . venv/bin/activate && pip install -r pip-requirements.txt --upgrade
ENV toolchain=GCC5
RUN . venv/bin/activate && python BaseTools/Edk2ToolsBuild.py -t "${toolchain}"

# Build options.
ENV common_build_opt="BLD_*_NETWORK_HTTP_ENABLE=1"
ENV tpm_build_opt="BLD_*_TPM1_ENABLE=1 BLD_*_TPM2_ENABLE=1"

# Build AARCH64.
ENV GCC5_AARCH64_PREFIX="/usr/bin/aarch64-linux-gnu-"
ENV build_target=RELEASE
ENV stuart_opts="-c ArmVirtPkg/PlatformCI/QemuBuild.py -a AARCH64 Target=${build_target} TOOL_CHAIN_TAG=${toolchain} ${common_build_opt}"
RUN . venv/bin/activate && stuart_setup ${stuart_opts} && stuart_update ${stuart_opts} && stuart_build ${stuart_opts}

# Build RiscV.
ENV GCC5_RISCV64_PREFIX="/opt/riscv/bin/riscv64-unknown-elf-"
ENV build_target=RELEASE
ENV stuart_opts="-c OvmfPkg/PlatformCI/QemuBuild.py -a RISCV64 Target=${build_target} TOOL_CHAIN_TAG=${toolchain} ${common_build_opt}"
RUN . venv/bin/activate && stuart_setup ${stuart_opts} && stuart_update ${stuart_opts} && stuart_build ${stuart_opts}

# Build IA32.
ENV stuart_opts="-c OvmfPkg/PlatformCI/PlatformBuild.py -a IA32 Target=${build_target} TOOL_CHAIN_TAG=${toolchain} ${common_build_opt} ${tpm_build_opt}"
RUN . venv/bin/activate && stuart_setup ${stuart_opts} && stuart_update ${stuart_opts} && stuart_build ${stuart_opts}

# Build X64.
ENV stuart_opts="-c OvmfPkg/PlatformCI/PlatformBuild.py -a X64 Target=${build_target} TOOL_CHAIN_TAG=${toolchain} ${common_build_opt} ${tpm_build_opt}"
RUN . venv/bin/activate && stuart_setup ${stuart_opts} && stuart_update ${stuart_opts} && stuart_build ${stuart_opts}

# Create the output bin dir.
ARG bin_dir
RUN mkdir "${bin_dir}"
RUN mkdir "${bin_dir}"/aarch64
RUN mkdir "${bin_dir}"/riscv64
RUN mkdir "${bin_dir}"/ia32
RUN mkdir "${bin_dir}"/x64

# Copy AARCH64 files to bin dir.
ENV aarch64_build="Build/ArmVirtQemu-AARCH64/${build_target}_${toolchain}"
RUN cp "${aarch64_build}/FV/QEMU_EFI.fd" "${bin_dir}"/aarch64/code.fd
RUN cp "${aarch64_build}/FV/QEMU_VARS.fd" "${bin_dir}"/aarch64/vars.fd
RUN cp "${aarch64_build}/AARCH64/Shell.efi" "${bin_dir}"/aarch64/shell.efi
# QEMU requires the AARCH64 files to be exactly 64MiB, so expand with zeroes.
RUN truncate --size=64MiB "${bin_dir}"/aarch64/code.fd
RUN truncate --size=64MiB "${bin_dir}"/aarch64/vars.fd

# Copy Riscv files to bin dir.
ENV riscv_build="Build/RiscVVirtQemu/${build_target}_${toolchain}"
RUN cp "${riscv_build}/FV/RISCV_VIRT_CODE.fd" "${bin_dir}"/riscv64/code.fd
RUN cp "${riscv_build}/FV/RISCV_VIRT_VARS.fd" "${bin_dir}"/riscv64/vars.fd
RUN cp "${riscv_build}/RISCV64/Shell.efi" "${bin_dir}"/riscv64/shell.efi
# QEMU requires the RISCV64 files to be exactly 32MiB, so expand with zeroes.
RUN truncate --size=32MiB "${bin_dir}"/riscv64/code.fd
RUN truncate --size=32MiB "${bin_dir}"/riscv64/vars.fd

# Copy IA32 files to bin dir.
ENV ia32_build="Build/OvmfIa32/${build_target}_${toolchain}"
RUN cp "${ia32_build}/FV/OVMF_CODE.fd" "${bin_dir}"/ia32/code.fd
RUN cp "${ia32_build}/FV/OVMF_VARS.fd" "${bin_dir}"/ia32/vars.fd
RUN cp "${ia32_build}/IA32/Shell.efi" "${bin_dir}"/ia32/shell.efi

# Copy X64 files to bin dir.
ENV x64_build="Build/OvmfX64/${build_target}_${toolchain}"
RUN cp "${x64_build}/FV/OVMF_CODE.fd" "${bin_dir}"/x64/code.fd
RUN cp "${x64_build}/FV/OVMF_VARS.fd" "${bin_dir}"/x64/vars.fd
RUN cp "${x64_build}/X64/Shell.efi" "${bin_dir}"/x64/shell.efi

# Create the compressed tarball of the bin dir.
RUN tar cf "${bin_dir}.tar" "${bin_dir}"
RUN xz "${bin_dir}.tar"
