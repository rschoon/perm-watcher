#!/bin/sh -x

cd `dirname $0`/..

WORKDIR=target/pkg
PKG=perm-watcher

if [ ! -z "$VERSION" ]; then
    VERSION="-$VERSION"
fi

mkdir -p dist $WORKDIR/$PKG

cp target/release/perm-watcher $WORKDIR/$PKG/

tar -cjf dist/perm-watcher-bin$VERSION.tar.bz2 -C $WORKDIR $PKG

rm -r $WORKDIR

