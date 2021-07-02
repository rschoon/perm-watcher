#!/bin/sh -e

if [ ! -z "$GPG_SIGNING_KEY" ]; then (cat <<END
$GPG_SIGNING_KEY
END
) | gpg --import; fi

codename=default
component=main-dev
if [ ! -z "$CI_COMMIT_TAG" ] || [ "$CI_COMMIT_BRANCH" == "hotfix" ]; then
    component=main
fi

deb-s3-upload-dir \
    --access-key-id="$AWS_ACCESS_KEY_ID" \
    --secret-access-key="$AWS_SECRET_ACCESS_KEY" \
    --sign="$GPG_SIGNING_KEY_ID" \
    --s3-region us-west-2 -b rschoon-deb --preserve-versions \
    -m $component -c $codename --prefix=tools dist/

deb-s3-clean \
    --access-key-id="$AWS_ACCESS_KEY_ID" \
    --secret-access-key="$AWS_SECRET_ACCESS_KEY" \
    --sign="$GPG_SIGNING_KEY_ID" \
    --s3-region us-west-2 -b rschoon-deb --prefix=tools \
    -a amd64 -m $component -c $codename \
    --keep-releases 5 --keep-releases-last 10 --keep-versions 20
