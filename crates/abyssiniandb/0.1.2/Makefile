
all: readme

readme: README.md

README.md: README.tpl src/lib.rs
	cargo readme > $@
