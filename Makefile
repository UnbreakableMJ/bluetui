# SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
# SPDX-License-Identifier: GPL-3.0-only
#
# Documentation build (The Steelbore Standard §8.2). The Rust code is built
# with Cargo; this Makefile builds the Texinfo manual in all three formats.

PROJECT = bluetui
TEXI    = doc/$(PROJECT).texi

.PHONY: all doc info html pdf clean

all: doc
doc: info html pdf

info: doc/$(PROJECT).info
doc/$(PROJECT).info: $(TEXI)
	makeinfo --output=$@ $(TEXI)

html: doc/$(PROJECT).html
doc/$(PROJECT).html: $(TEXI)
	makeinfo --html --no-split --output=$@ $(TEXI)

pdf: doc/$(PROJECT).pdf
doc/$(PROJECT).pdf: $(TEXI)
	texi2pdf --quiet --output=$@ $(TEXI)

clean:
	rm -f doc/$(PROJECT).info doc/$(PROJECT).html doc/$(PROJECT).pdf
