release-patch:
	cargo release --no-publish --no-push -x patch

release-minor:
	cargo release --no-publish --no-push -x minor
