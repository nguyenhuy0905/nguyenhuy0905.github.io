CMD:=python3 scripts/ssg.py
CONTENT_FILES:=index.html blog/index.html

all: index.html blog/index.html

index.html: template/header.html template/footer.html template/index.html template/index/content.html
	$(CMD) template/index.html index.html

blog/index.html: template/header.html template/footer.html template/blog/index.html
	$(CMD) template/blog/index.html blog/index.html

blog/dev: header footer template/blog/dev/1-database.html
	$(CMD) template/blog/dev/1-database.html blog/dev/1-database.html

.PHONY: format
format:
	prettier -w $(CONTENT_FILES)
