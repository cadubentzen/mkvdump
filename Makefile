release-patch:
	cargo release --workspace --no-publish --no-push -x patch

release-minor:
	cargo release --workspace --no-publish --no-push -x minor
