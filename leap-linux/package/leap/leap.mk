################################################################################
#
# leap-server
#
################################################################################
LEAP_VERSION = 0.1.0
LEAP_SITE = ../../../result/bin
LEAP_LICENSE = MIT

define LEAP_BUILD_CMDS
endef

define LEAP_INSTALL_STAGING_CMDS
endef

define LEAP_INSTALL_TARGET_CMDS
    $(INSTALL) -D -m 0755 $(@D)/leap $(TARGET_DIR)/usr/bin/leap
endef

$(eval $(generic-package))
