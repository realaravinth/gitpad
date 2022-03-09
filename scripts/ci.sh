#!/bin/bash
# Used in CI workflow: install Zola binary from GitHub
# Copyright © 2021 Aravinth Manivannan <realaravinth@batsense.net>
# 
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as
# published by the Free Software Foundation, either version 3 of the
# License, or (at your option) any later version.
# 
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU Affero General Public License for more details.
# 
# You should have received a copy of the GNU Affero General Public License
# along with this program.  If not, see <http://www.gnu.org/licenses/>.

set -euo pipefail

readonly TARBALL=zola.tar.gz
readonly SOURCE="https://github.com/getzola/zola/releases/download/v0.15.3/zola-v0.15.3-x86_64-unknown-linux-gnu.tar.gz"

readonly BIN_PATH=bin
readonly BIN=$BIN_PATH/zola

readonly DIST=public
readonly WEBSITE=website

cd $WEBSITE

help() {
	cat << EOF
ci.sh: CI build script
USAGE:
    ci.sh <options>
OPTIONS:
  b   build     build website
  c   clean     clean dependencies and build artifacts
  h   help      print this help menu
  i   install   install build dependencies
  u   url       make urls relative
  z   zola      invoke zola
EOF
}

check_arg(){
	if [ -z $1 ]
	then
		help
		exit 1
	fi
}

match_arg() {
	if [ $1 == $2 ] || [ $1 == $3 ]
	then
		return 0
	else
		return 1
	fi
}

download() {
	echo "Downloading Zola"
	wget --quiet --output-document=$TARBALL $SOURCE
	tar -xvzf $TARBALL > /dev/null
	rm $TARBALL
	echo "Downloaded zola into $BIN" 
}

init() {
	if [ ! -d $BIN_PATH ]
	then
		mkdir $BIN_PATH
	fi

	if [ ! -f $BIN ]
	then
		cd $BIN_PATH
		download
	fi
}

run() {
	$BIN "${@:1}"
}

build() {
	run build
}

no_absolute_url() {
	sed -i 's/https:\/\/batsense.net//g' $(find public -type f | grep html)
}

clean() {
	rm -rf $BIN_PATH || true
	rm -rf $DIST || true
	echo "Workspace cleaned"
}

check_arg $1

if match_arg $1 'i' 'install'
then
	init
elif match_arg $1 'c' 'clean'
then
	clean
elif match_arg $1 'b' 'build'
then
	build
elif match_arg $1 'h' 'help'
then
	help
elif match_arg $1 'u' 'url'
then 
	no_absolute_url
elif match_arg $1 'z' 'zola'
then
	$BIN "${@:3}"
else
	echo "Error: $1 is not an option"
	help
	exit 1
fi

exit 0
