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
    uuid-dev

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

# Build AARCH64.
ENV GCC5_AARCH64_PREFIX="/usr/bin/aarch64-linux-gnu-"
# Note: when updating to the next release, replace PlatformBuild.py with QemuBuild.py
ENV build_target=RELEASE
ENV stuart_opts="-c ArmVirtPkg/PlatformCI/PlatformBuild.py -a AARCH64 Target=${build_target} TOOL_CHAIN_TAG=${toolchain}"
RUN . venv/bin/activate && stuart_setup ${stuart_opts} && stuart_update ${stuart_opts} && stuart_build ${stuart_opts}

# Build IA32.
ENV stuart_opts="-c OvmfPkg/PlatformCI/PlatformBuild.py -a IA32 Target=${build_target} TOOL_CHAIN_TAG=${toolchain} BLD_*_TPM1_ENABLE=1 BLD_*_TPM2_ENABLE=1"
RUN . venv/bin/activate && stuart_setup ${stuart_opts} && stuart_update ${stuart_opts} && stuart_build ${stuart_opts}

# Build X64.
ENV stuart_opts="-c OvmfPkg/PlatformCI/PlatformBuild.py -a X64 Target=${build_target} TOOL_CHAIN_TAG=${toolchain} BLD_*_TPM1_ENABLE=1 BLD_*_TPM2_ENABLE=1"
RUN . venv/bin/activate && stuart_setup ${stuart_opts} && stuart_update ${stuart_opts} && stuart_build ${stuart_opts}

# Create the output bin dir.
ARG bin_dir
RUN mkdir "${bin_dir}"
RUN mkdir "${bin_dir}"/aarch64
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
