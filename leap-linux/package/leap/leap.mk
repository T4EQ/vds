################################################################################
#
# leap-server
#
################################################################################
LEAP_VERSION = 0.1.0
LEAP_SITE = $(BR2_EXTERNAL_LEAP_PATH)/package/leap/src
LEAP_SITE_METHOD = local
LEAP_LICENSE = MIT

define LEAP_BUILD_CMDS
endef

define LEAP_INSTALL_STAGING_CMDS
endef

define LEAP_INSTALL_TARGET_CMDS
    $(INSTALL) -D -m 0755 $(@D)/leap-server $(TARGET_DIR)/usr/bin/leap-server
endef

$(eval $(generic-package))
