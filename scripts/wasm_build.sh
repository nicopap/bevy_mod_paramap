#!/bin/bash
set -e

# Install wasm-bindgen with `cargo install wasm-bindgen-cli`.
# Pass --run option to run after build (uses python).
# Files in OutDir is everything needed to run the web page.

function build_example {
	Features="$2"
	Example="$1"
	ReleaseFlag="--example $Example $Features"


	cargo build --no-default-features \
		--profile wasm-release \
		$ReleaseFlag --target wasm32-unknown-unknown

	WasmFile="$(cargo metadata --format-version 1 | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')/wasm32-unknown-unknown/$BuildDir/$Example.wasm"

	if [ ! -e "$WasmFile" ]; then
		echo "Script is borken, it expects file to exist: $WasmFile"
		exit 1
	fi

	BINDGEN_EXEC_PATH="${CARGO_HOME:-~/.cargo}/bin/wasm-bindgen"

	if [ ! -e "$BINDGEN_EXEC_PATH" ] ; then
	    echo "Please install wasm-bindgen, cannot generate the wasm output without it"
	    exit 1
	fi

	$BINDGEN_EXEC_PATH \
		--no-typescript \
		--out-dir "$OutDir" \
		--target web \
		"$WasmFile"

	if [ -e $(which wasm-opt) ] ; then
		BindgenOutput="$OutDir/${Example}_bg.wasm"
		echo "Applying wasm-opt optimizations"
		echo "before: $(wc -c "$BindgenOutput")"
		wasm-opt -Oz --output "$BindgenOutput.post-processed" "$BindgenOutput"
		echo "after : $(wc -c "$BindgenOutput.post-processed")"
		mv "$BindgenOutput.post-processed" "$BindgenOutput"
	else
		echo "Continuing without wasm-opt, it is highly recommended that you"
		echo "install it, it has been known to divide by two wasm files size"
	fi
}

if [ ! -e .git ]; then
	echo "Must be run from repository root"
	exit 1
fi

OutDir=github_pages

BuildDir="wasm-release/examples"

if [ ! -e target ] ; then
    mkdir target
fi

[ ! -e "$OutDir" ] || rm -r "$OutDir"

build_example "earth3d" "--features inspector-def"
build_example "cube" ""

cp scripts/demo_page.html "$OutDir/index.html"
cp scripts/wasm_build.html "$OutDir/earth3d.html"
cp scripts/wasm_build.html "$OutDir/cube.html"
sed -i "s/main.js/earth3d.js/g" "$OutDir/earth3d.html"
sed -i "s/main.js/cube.js/g" "$OutDir/cube.html"

cp -r assets "$OutDir/assets"
