#!/bin/sh -x

cd `dirname $0`/..

WORKDIR=target/pkg
PKG=perm-watcher

if [ -z "$CI_COMMIT_TAG" ] || [ ! -z "$DEV_SNAPSHOT" ]; then
    REL=`date +"%Y%m%d%H%M%S"`
    VERSION=`cargo metadata --format-version 1 | jq -r '.packages[] | select(.name=="perm-watcher") | .version'`
    DEB_ARGS="--deb-version "`echo $VERSION | sed "s/-dev/~0dev$REL/"`
fi

cargo deb $DEB_ARGS

if [ ! -z "$VERSION" ]; then
    VERSION_SUFFIX="-$VERSION"
fi

mkdir -p dist $WORKDIR/$PKG

cp target/debian/*.deb dist/
cp target/release/perm-watcher $WORKDIR/$PKG/
tar -cjf dist/perm-watcher-bin$VERSION_SUFFIX.tar.bz2 -C $WORKDIR $PKG

rm -r $WORKDIR

