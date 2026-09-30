CMD:=python3 scripts/ssg.py
CONTENT_FILES:=index.html blog/index.html

DEV_BLOG_NAMES:=index 1-database
DEV_BLOG_FILES:=$(patsubst %,blog/dev/%.html,$(DEV_BLOG_NAMES))
DEV_BLOG_INPUTS:=$(patsubst %,template/%,$(DEV_BLOG_FILES))

all: index.html blog/index.html

index.html: template/header.html template/footer.html template/index.html template/index/content.html
	$(CMD) template/index.html index.html

blog/index.html: template/header.html template/footer.html template/blog/index.html blog/dev
	$(CMD) template/blog/index.html blog/index.html

blog/dev: $(DEV_BLOG_FILES)

$(DEV_BLOG_FILES): %: template/% template/header.html template/footer.html
	$(CMD) $< $@

# blog/dev: template/header.html template/footer.html \
# 	template/blog/dev/index.html \
# 	template/blog/dev/1-database.html
# 	$(CMD) template/blog/dev/index.html blog/dev/index.html
# 	$(CMD) template/blog/dev/1-database.html blog/dev/1-database.html

.PHONY: format
format:
	prettier -w $(CONTENT_FILES)
