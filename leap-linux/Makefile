BUILDROOT_PATH ?= ./buildroot

OUT_DIR ?= $(PWD)/output

DEFCONFIG_NAME ?= leap_defconfig
DEFCONFIG_PATH ?= $(PWD)/configs/$(DEFCONFIG_NAME)

build:
	make -C $(BUILDROOT_PATH) BR2_EXTERNAL=$(PWD) O=$(OUT_DIR)

clean:
	rm -rf $(OUT_DIR)

defconfig:
	make -C $(BUILDROOT_PATH) BR2_EXTERNAL=$(PWD) O=$(OUT_DIR) $(DEFCONFIG_NAME)

menuconfig:
	make -C $(BUILDROOT_PATH) BR2_EXTERNAL=$(PWD) O=$(OUT_DIR) menuconfig

savedefconfig:
	make -C $(BUILDROOT_PATH) BR2_EXTERNAL=$(PWD) O=$(OUT_DIR) BR2_DEFCONFIG=$(DEFCONFIG_PATH) savedefconfig
