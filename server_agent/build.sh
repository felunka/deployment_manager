#!/usr/bin/env bash

cargo build --release

rm ../public/agent_bundle.zip

mkdir ../public/api_host
cp ./target/release/server_agent ../public/api_host

cd ../public
zip -r agent_bundle.zip api_host/

rm -r ../public/api_host
