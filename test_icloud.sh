#!/bin/bash
OBJC2_LIB=$(ls target/release/deps/libobjc2-*.rlib | grep -v "\.rmeta" | head -n 1)
FOUND_LIB=$(ls target/release/deps/libobjc2_foundation-*.rlib | grep -v "\.rmeta" | head -n 1)
rustc --edition 2021 test_icloud.rs -L dependency=target/release/deps --extern objc2=$OBJC2_LIB --extern objc2_foundation=$FOUND_LIB && ./test_icloud
