#!/usr/bin/bash

LINUX_REPO=linux-cloud-hypervisor

if [ ! -d $LINUX_REPO ]
then
    git clone --depth 1 "https://github.com/cloud-hypervisor/linux.git" -b "ch-6.2" $LINUX_REPO
fi

pushd $LINUX_REPO
cp ../../scripts/alpine_config ./alpine_config
KCFLAGS="-Wa,-mx86-used-note=no" KCONFIG_CONFIG="alpine_config" KBUILD_OUTPUT="/home/hugo/Bureau/cours/rust_sor/sealci/dumper/vm/linux-cloud-hypervisor" make vmlinux -j `nproc`
popd
