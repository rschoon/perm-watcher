#!/bin/sh -x

cd `dirname $0`/..

dist="${DISTDIR:-dist}"
WORKDIR=target/pkg
PKG=perm-watcher

VERSION=`cargo metadata --format-version 1 | jq -r '.packages[] | select(.name=="'"$PKG"'") | .version' | sed 's/-\(dev\|alpha\|beta\)/~\1/'`
if [ -z "$CI_COMMIT_TAG" ] || [ -n "$DEV_SNAPSHOT" ]; then
    REL=`date +"%Y%m%d%H%M%S"`
    if ! echo "$VERSION" | grep -q "dev"; then
        VERSION="$VERSION-dev"
    fi
    DEB_ARGS="--deb-version $VERSION.$REL"
fi

cargo deb $DEB_ARGS

if [ ! -z "$VERSION" ]; then
    VERSION_SUFFIX="-$VERSION"
fi

mkdir -p $dist $WORKDIR/$PKG

cp target/debian/*.deb $dist/
cp target/release/perm-watcher $WORKDIR/$PKG/
tar -cjf $dist/perm-watcher-bin$VERSION_SUFFIX.tar.bz2 -C $WORKDIR $PKG

rm -r $WORKDIR

