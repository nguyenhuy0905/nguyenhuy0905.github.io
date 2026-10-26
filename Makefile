CMD:=python3 scripts/ssg.py
CONTENT_FILES:=index.html blog/index.html

DEV_BLOG_NAMES:=index 1-database 2-ssg
DEV_BLOG_FILES:=$(patsubst %,build/blog/dev/%.html,$(DEV_BLOG_NAMES))
DEV_BLOG_INPUTS:=$(patsubst build/%,template/%,$(DEV_BLOG_FILES))

all: build/index.html build/blog/index.html build/style.css

build/index.html: template/header.html template/footer.html template/index.html template/index/content.html
	$(CMD) template/index.html build/index.html

build/blog/index.html: template/header.html template/footer.html template/blog/index.html build/blog/dev
	$(CMD) template/blog/index.html build/blog/index.html

build/blog/dev: $(DEV_BLOG_FILES)

$(DEV_BLOG_FILES): build/%: template/% template/header.html template/footer.html
	$(CMD) $< $@

build/style.css: style.css
	cp $< $@

.PHONY: format nuke
format:
	prettier -w $(CONTENT_FILES)

nuke:
	rm -rf build
