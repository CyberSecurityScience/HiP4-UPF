# Control plane
As mentioned in the paper there are two parts: south controller built in C++ and north controller built in rust \
## Some notes
We use name `MAID` or `MA_ID` (Match Action ID) in controller implementation to refer to `PDR_ID` in the paper
## South controller
Source code is available in `controller/south_controller` 
### Building south controller
You need Intel BF-SDE 9.11.2 or later and gcc-11 or later (C++20 is required) 
1. Goto `build/`
2. `cmake .. -DCMAKE_BUILD_TYPE=Release`
3. `make -j4`
### Running south controller
After building P4 code, putting the resultant folder in `$SDE_INSTALL` as `$SDE_INSTALL/upf.tofino` \
Goto `build/` \
Switch to root user \
Run `./upf_driver --install-dir $SDE_INSTALL --conf-file $SDE_INSTALL/upf.tofino/upf.conf --ucli` 
### Cleanup
Use Ctrl+C to stop south controller \
And run `rm /tmp/upf-*` to clean sockets use to communicate between south and north controller

## North Controller
Source code can be found in `controller/north_controller/src/nf/upf` \
You need rust musl toolchain, install it using `rustup toolchain install stable-x86_64-unknown-linux-musl` \
You also need OpenSSL compiled using musl, you can install using [this tutorial](https://qiita.com/liubin/items/6c94f0b61f746c08b74c)
### Building north controller
Run `build.sh` in north controller folder
### Running north controller
After building you can find a statically linked UPF binary in `target/x86_64-unknown-linux-musl/release/upf` \
Copy this file and `upf-config-netberg-710.yaml` to P4 switch and run it using `sudo -E ./upf --config upf-config-netberg-710.yaml --ip 0.0.0.0`
