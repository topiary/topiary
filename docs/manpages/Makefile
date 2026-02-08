SOURCES := book.toml $(shell find src -type f 2>/dev/null)

NAME := topiary
BUILD_DIR := man
BUILD_PREFIX := $(BUILD_DIR)/$(NAME)

MAN_DIR ?= "${HOME}/.local/share/man"

# NOTE We assume that mdbook-manmunge is available in the $PATH. This
# Makefile does not build mdbook-manmunge from source.
MDBOOK_MANMUNGE := mdbook-manmunge

all: $(BUILD_PREFIX).1.gz

$(BUILD_PREFIX).1.gz: $(BUILD_PREFIX).man
	$(MDBOOK_MANMUNGE) post-process < "$<" \
	| gzip --stdout \
	> "$@"

$(BUILD_PREFIX).man: $(SOURCES)
	mdbook build

install: $(MAN_DIR)/man1/$(NAME).1.gz

$(MAN_DIR)/man1/$(NAME).1.gz: $(BUILD_PREFIX).1.gz $(MAN_DIR)/man1
	install \
	  -m 0644 \
	  "$<" \
	  "$@"

$(MAN_DIR)/man1:
	mkdir -p "$@"

uninstall:
	rm -rf "$(MAN_DIR)/man1/$(NAME).1.gz"

clean:
	rm -rf "$(BUILD_DIR)"

.PHONY: all install uninstall clean
