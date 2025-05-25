# SPDX-License-Identifier: MPL-2.0

include ../test_common.mk

EXTRA_C_FLAGS := -static

ifeq ($(ARCH), x86_64)
EXTRA_C_FLAGS += -mno-sse
endif
