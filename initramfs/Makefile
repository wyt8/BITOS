# SPDX-License-Identifier: MPL-2.0

BINARY_CACHE_DIR := /opt/linux_binary_cache
VDSO_DIR := ../target
VDSO_LIB := $(VDSO_DIR)/vdso64.so
MKFILE_PATH := $(abspath $(lastword $(MAKEFILE_LIST)))
CUR_DIR := $(patsubst %/,%,$(dir $(MKFILE_PATH)))
ATOMIC_WGET := $(CUR_DIR)/../tools/atomic_wget.sh
BUILD_DIR := $(CUR_DIR)/build
TMP_DIR := $(CUR_DIR)/tmp
TOOLCHAIN_DIR := $(CUR_DIR)/toolchain
INITRAMFS := $(BUILD_DIR)/initramfs
INITRAMFS_FILELIST := $(BUILD_DIR)/initramfs.filelist
BENCHMARK := ""
BUILD_SYSCALL_TEST := 0
INITRAMFS_SKIP_GZIP ?= 1
ifeq ($(INITRAMFS_SKIP_GZIP),1)
INITRAMFS_IMAGE := $(BUILD_DIR)/initramfs.cpio
else
INITRAMFS_IMAGE := $(BUILD_DIR)/initramfs.cpio.gz
endif
EXT2_IMAGE := $(BUILD_DIR)/ext2.img
EXFAT_IMAGE := $(BUILD_DIR)/exfat.img
INITRAMFS_EMPTY_DIRS := \
	$(INITRAMFS)/root \
	$(INITRAMFS)/tmp \
	$(INITRAMFS)/opt \
	$(INITRAMFS)/proc \
	$(INITRAMFS)/dev \
	$(INITRAMFS)/sys \
	$(INITRAMFS)/ext2 \
	$(INITRAMFS)/exfat
# INITRAMFS_ALL_DIRS := \
# 	$(INITRAMFS)/etc \
# 	$(INITRAMFS)/lib/x86_64-linux-gnu \
# 	$(INITRAMFS)/lib64 \
# 	$(INITRAMFS)/bin \
# 	$(INITRAMFS)/sbin \
# 	$(INITRAMFS)/usr/bin \
# 	$(INITRAMFS)/usr/local \
# 	$(INITRAMFS)/benchmark \
# 	$(INITRAMFS_EMPTY_DIRS)
INITRAMFS_ALL_DIRS := \
	$(INITRAMFS)/etc \
	$(INITRAMFS)/bin \
	$(INITRAMFS)/sbin \
	$(INITRAMFS)/usr/bin \
	$(INITRAMFS)/usr/sbin \
	$(INITRAMFS)/lib/riscv64-linux-gnu \
	$(INITRAMFS)/lib64 \
	$(INITRAMFS)/usr/lib/riscv64-linux-gnu \
	$(INITRAMFS)/usr/lib64 \
	$(INITRAMFS_EMPTY_DIRS)
# INITRAMFS_ALL_DIRS := \
#  	$(INITRAMFS)/etc \
# 	$(INITRAMFS)/bin \
# 	$(INITRAMFS)/usr/bin \
# 	$(INITRAMFS_EMPTY_DIRS)

# Include test as target if BENCHMARK is not set.
ifeq ($(BENCHMARK), none)
INITRAMFS_ALL_DIRS += $(INITRAMFS)/test
endif

SYSCALL_TEST_DIR := $(INITRAMFS)/opt/syscall_test

.PHONY: all
all: build

$(VDSO_LIB): | $(VDSO_DIR) $(BINARY_CACHE_DIR)/vdso64.so
	@# TODO: use a custom compiled vdso.so file in the future.
	@cp $(BINARY_CACHE_DIR)/vdso64.so $@
	
$(VDSO_DIR):
	@mkdir -p $@
	
$(BINARY_CACHE_DIR)/vdso64.so:
	@mkdir -p $(BINARY_CACHE_DIR)
	@$(ATOMIC_WGET) $@ "https://raw.githubusercontent.com/asterinas/linux_vdso/2a6d2db/vdso64.so"

$(INITRAMFS)/usr/local:
	@mkdir -p $@
	@cp -r /usr/local/nginx $@
	@cp -r /usr/local/redis $@
	@cp -r /usr/local/memcached $@

$(INITRAMFS)/benchmark: | $(INITRAMFS)/benchmark/bin
	@cp -rf $(CUR_DIR)/benchmark/* $@

$(INITRAMFS)/benchmark/bin:
	@mkdir -p $@
	@cp /usr/local/benchmark/sysbench/bin/sysbench $@
	@cp /usr/local/benchmark/iperf/bin/iperf3 $@
	@cp /usr/local/benchmark/membench/membench $@
	@cp /usr/local/benchmark/iozone $@
	@cp -r /usr/local/benchmark/unixbench $@
	@cp -r /usr/local/benchmark/lmbench $@
	@cp /usr/local/benchmark/fio/bin/fio $@
	@cp /usr/local/benchmark/sqlite-speedtest1 $@
	@cp /usr/local/leveldb/benchmark/db_bench $@
	@mv $@/db_bench $@/db_bench_leveldb
	@cp /usr/local/leveldb/benchmark/db_bench_sqlite3 $@
	@cp /usr/local/benchmark/hackbench $@
	@cp /usr/local/benchmark/schbench $@

.PHONY: $(SYSCALL_TEST_DIR)
$(SYSCALL_TEST_DIR):
	@$(MAKE) --no-print-directory -C syscall_test

.PHONY: $(INITRAMFS_IMAGE)
$(INITRAMFS_IMAGE): $(INITRAMFS_FILELIST)
	@if ! cmp -s $(INITRAMFS_FILELIST) $(INITRAMFS_FILELIST).previous || ! test -f $@; then \
		echo "Generating the initramfs image..."; \
		cp -f $(INITRAMFS_FILELIST) $(INITRAMFS_FILELIST).previous; \
		( \
			cd $(INITRAMFS); \
			# `$(INITRAMFS_FILELIST)` contains files' last modification \
			# time in the first column, and files' relative path to \
			# `$(INITRAMFS)` in the second column. This prunes the first \
			# column and passes the second column to `cpio`. \
			cut -d " " -f 2- $(INITRAMFS_FILELIST) | \
				cpio -o -H newc | \
					if [ "$(INITRAMFS_SKIP_GZIP)" != 1 ]; then \
						gzip; \
					else \
						cat; \
					fi \
		) > $@; \
	fi

.PHONY: $(INITRAMFS_FILELIST)
# If the BUILD_SYSCALL_TEST variable is set, we should depend on the
# sub make output to do incremental building.
ifeq ($(BUILD_SYSCALL_TEST), 1)
$(INITRAMFS_FILELIST): | $(INITRAMFS_ALL_DIRS) $(SYSCALL_TEST_DIR)
else
$(INITRAMFS_FILELIST): | $(INITRAMFS_ALL_DIRS)
endif
	@(cd $(INITRAMFS); find . -printf "%T@ %p\n") > $(INITRAMFS_FILELIST)

$(EXT2_IMAGE):
	@dd if=/dev/zero of=$(EXT2_IMAGE) bs=2G count=1
	@mke2fs $(EXT2_IMAGE)

$(EXFAT_IMAGE):
	@fallocate -l 64M $(EXFAT_IMAGE)
	@mkfs.exfat $(EXFAT_IMAGE)

.PHONY: build
build: $(INITRAMFS_IMAGE) $(EXT2_IMAGE)
# build: $(INITRAMFS_IMAGE) $(EXT2_IMAGE) $(EXFAT_IMAGE)

.PHONY: format
format:
	@$(MAKE) --no-print-directory -C apps format

.PHONY: check
check:
	@$(MAKE) --no-print-directory -C apps check

.PHONY: clean
clean:
	@rm -rf $(BUILD_DIR)
	@rm -rf $(TMP_DIR)
	@rm -rf $(TOOLCHAIN_DIR)

.PHONY: test
test: $(INITRAMFS)/bin

# =========================== ToolChain ===============================


# Download and extract the prebuilt riscv64 toolchain into a fixed name
$(TOOLCHAIN_DIR)/riscv64:
	@mkdir -p $(TOOLCHAIN_DIR)
	@$(ATOMIC_WGET) $(TOOLCHAIN_DIR)/toolchain.tar.xz https://toolchains.bootlin.com/downloads/releases/toolchains/riscv64-lp64d/tarballs/riscv64-lp64d--glibc--stable-2024.05-1.tar.xz
	@tar -xJf $(TOOLCHAIN_DIR)/toolchain.tar.xz -C $(TOOLCHAIN_DIR)
	@mv $(TOOLCHAIN_DIR)/riscv64-lp64d--glibc--stable-2024.05-1 $(TOOLCHAIN_DIR)/riscv64


# =========================== BusyBox ===============================


# Clone the source code of busybox.
$(TMP_DIR)/busybox:
	@git clone git://busybox.net/busybox.git $(TMP_DIR)/busybox

# Build the busybox.
$(TMP_DIR)/busybox/_install: $(TMP_DIR)/busybox $(TOOLCHAIN_DIR)/riscv64
	@make --no-print-directory -C $(TMP_DIR)/busybox defconfig CROSS_COMPILE=$(TOOLCHAIN_DIR)/riscv64/bin/riscv64-linux- ARCH=riscv64
	@# Modify the busybox config file to enable statically linked
	@sed -i 's/^# CONFIG_STATIC is not set/CONFIG_STATIC=y/' $(TMP_DIR)/busybox/.config
	@make --no-print-directory -C $(TMP_DIR)/busybox CROSS_COMPILE=$(TOOLCHAIN_DIR)/riscv64/bin/riscv64-linux- ARCH=riscv64
	@make --no-print-directory -C $(TMP_DIR)/busybox install CROSS_COMPILE=$(TOOLCHAIN_DIR)/riscv64/bin/riscv64-linux- ARCH=riscv64 CFLAGS="-fno-stack-protector"


# =========================== InitRamFs ===============================

# Make necessary directories.
$(INITRAMFS_EMPTY_DIRS):
	@mkdir -p $@

$(INITRAMFS)/bin: $(TMP_DIR)/busybox/_install
	@mkdir -p $@
	@cp -P -r $(TMP_DIR)/busybox/_install/bin/* $@

$(INITRAMFS)/sbin: $(TMP_DIR)/busybox/_install $(TOOLCHAIN_DIR)/riscv64
	@cp -r $(TMP_DIR)/busybox/_install/sbin $@
	@cp -r $(TOOLCHAIN_DIR)/riscv64/riscv64-buildroot-linux-gnu/sysroot/sbin/* $@

$(INITRAMFS)/usr/bin: $(TMP_DIR)/busybox/_install $(TOOLCHAIN_DIR)/riscv64
	@mkdir -p $(INITRAMFS)/usr
	@cp -P -r $(TMP_DIR)/busybox/_install/usr/bin $@
	@cp -P -r $(TOOLCHAIN_DIR)/riscv64/riscv64-buildroot-linux-gnu/sysroot/usr/bin/* $@

$(INITRAMFS)/usr/sbin: $(TMP_DIR)/busybox/_install $(TOOLCHAIN_DIR)/riscv64
	@mkdir -p $(INITRAMFS)/usr
	@cp -r $(TMP_DIR)/busybox/_install/usr/sbin $@
	@cp -r $(TOOLCHAIN_DIR)/riscv64/riscv64-buildroot-linux-gnu/sysroot/usr/sbin/* $@

$(INITRAMFS)/lib/riscv64-linux-gnu: $(TOOLCHAIN_DIR)/riscv64
	@mkdir -p $@
	@cp -r $(TOOLCHAIN_DIR)/riscv64/riscv64-buildroot-linux-gnu/sysroot/lib/* $@

$(INITRAMFS)/lib64: $(TOOLCHAIN_DIR)/riscv64
	@mkdir -p $@
	@cp -L $(TOOLCHAIN_DIR)/riscv64/riscv64-buildroot-linux-gnu/sysroot/lib64/ld-linux-riscv64-lp64d.so.1 $@

$(INITRAMFS)/usr/lib/riscv64-linux-gnu: $(TOOLCHAIN_DIR)/riscv64
	@mkdir -p $@
	@cp -r -L $(TOOLCHAIN_DIR)/riscv64/riscv64-buildroot-linux-gnu/sysroot/usr/lib/* $@

$(INITRAMFS)/usr/lib64: $(TOOLCHAIN_DIR)/riscv64
	@mkdir -p $@
	@cp -r $(TOOLCHAIN_DIR)/riscv64/riscv64-buildroot-linux-gnu/sysroot/usr/lib64/* $@

$(INITRAMFS)/test: $(TOOLCHAIN_DIR)/riscv64
	@$(MAKE) --no-print-directory -C apps CC=$(TOOLCHAIN_DIR)/riscv64/bin/riscv64-linux-gcc

$(INITRAMFS)/etc:
	@mkdir -p $@
	@cp -r $(CUR_DIR)/etc/* $@
	@cp -r $(TOOLCHAIN_DIR)/riscv64/riscv64-buildroot-linux-gnu/sysroot/etc/* $@