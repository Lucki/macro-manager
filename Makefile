# Makefile for mod-manager

# Define installation directories
PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin
LIBDIR = $(PREFIX)/lib
SHAREDIR = $(PREFIX)/share
ZSHDIR = $(SHAREDIR)/zsh/site-functions

# Define files to install
BIN_FILE = macro-manager
LIB_FILE = libmacro_manager.so
ZSH_FILE = _macro-manager

# Targets
.PHONY: build install test clean

build:
	@echo "Building macro-manager…"
	@cargo build --release

install:
	@echo "Installing files…"
	install -D -m 755 target/release/$(BIN_FILE) $(DESTDIR)$(BINDIR)/$(BIN_FILE)
	install -D -m 644 target/release/$(LIB_FILE) $(DESTDIR)$(LIBDIR)/$(LIB_FILE)
	install -D -m 644 dist/$(ZSH_FILE) $(DESTDIR)$(ZSHDIR)/$(ZSH_FILE)

test:
	@echo "Testing…"
	@cargo test

clean:
	@echo "Cleaning up…"
	@cargo clean
