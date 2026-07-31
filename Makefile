CMD:=python3 scripts/ssg.py
CONTENT_FILES:=index.html blog/index.html

all: index blog

index: index.html

index.html: header footer template/index.html template/index/content.html
	$(CMD) template/index.html index.html

blog: blog/index.html

blog/index.html: header footer template/blog/index.html
	$(CMD) template/blog/index.html blog/index.html

header: template/header.html meta

footer: template/footer.html meta

meta: template/meta.html

.PHONY: format
format:
	prettier -w $(CONTENT_FILES)
