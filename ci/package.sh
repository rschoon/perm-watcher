#!/bin/sh -ex

cd `dirname $0`/..

dist="${DISTDIR:-dist}"

pkg=perm-watcher
VERSION=`cargo metadata --format-version 1 | jq -r '.packages[] | select(.name=="'"$pkg"'") | .version' | sed 's/-\(dev\|alpha\|beta\)/~\1/'`
if [ "$REF_TYPE" != "tag" ] || [ -n "$DEV_SNAPSHOT" ]; then
    REL=`date +"%Y%m%d%H%M%S"`
    if ! echo "$VERSION" | grep -q "dev"; then
        VERSION="$VERSION-dev"
    fi
    DEB_ARGS="--deb-version $VERSION.$REL"
fi

cargo deb $DEB_ARGS

mkdir -p $dist
cp target/debian/*.deb $dist/
