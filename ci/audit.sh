#!/bin/sh -e

cargo audit "$@" $AUDIT_ARGS
