#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

PWD="$(dirname $(readlink -f -- $0))"
DIST="$PWD/dist"
VERSION="1.28"
OS="$(uname -s)"
ARCH="$(uname -m)"

function help() {
	echo "Usage: $0 <flags>"
	echo
	echo "flags:"
	echo "  --version  hledger version to install.       (default: $VERSION)"
	echo "  --dist     directory to install binaries to. (default: $DIST)"
	echo "  --os       os name to install binaries for.  (default: $OS)"
	echo "  --help     display this message."
}

function error() {
	echo "error: $@"
	echo
	help
	exit 1
}

function info() {
	echo "$@"
}

while [[ $# -gt 0 ]]; do
	case "$1" in
	--help)
		help
		exit 1
		;;
	--version)
		VERSION="$2"
		shift
		shift
		;;
	--dist)
		DIST="$(readlink -f $2)"
		shift
		shift
		;;
	--os)
		OS="$2"
		shift
		shift
		;;
	*)
		error "unknown flag $1"
		;;
	esac
done

info "version: $VERSION"
info "dist: $DIST"

function download() {
	info "downloading $@"
	local http_code=$(curl --silent --remote-name --location --write-out "%{http_code}" "$@")
	if [[ ${http_code} -ne 200 ]]; then
		error "GET $@: $http_code"
	fi
}

case "$OS" in
Darwin)
	TMP_DIR=$(mktemp -d -t ci-XXXXXXXXXX)
	pushd "$TMP_DIR"
	download "https://github.com/simonmichael/hledger/releases/download/$VERSION/hledger-mac-x64.zip"
	unzip -o "hledger-mac-x64.zip"
	mkdir -p "$DIST"
	tar xvf "hledger-mac-x64.tar"
	mv "hledger-web" "$DIST/hledger-web-aarch64-apple-darwin"
	popd
	rm -rf $TMP_DIR
	;;
*)
	error "$OS unsupported"
	;;
esac
