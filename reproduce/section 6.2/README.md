# Reproducing result in Table 4 for open5gs/free5gc/hip4-upf
## Hardware requirements
2 servers connected together, one (server A) running `mock-smf-comprehensive`, the other one (server B) running open5gs/free5gc \
In addition to server A also connected to the Tofino switch
## Setting up open5gs/free5gc
### open5gs
Instruction for building open5gs can be found in https://open5gs.org/open5gs/docs/guide/02-building-open5gs-from-sources/ \
You don't need to install mongodb or install open5gs on your system, after building you can use its UPF binary at `install/bin/open5gs-upfd` \
After building replace `install/etc/open5gs/upf.yaml` with the provided `upf.yaml`, change to IP address in line 13 and 19 to the IP address of server B and change the filepath in line 2 \

Running open5gs UPF
`sudo ./install/bin/open5gs-upfd` \
This will take a few minutes, after start up you should be able to see
```
[app] INFO: UPF initialize...done (../src/upf/app.c:31)
```

### free5gc
Download and build gtp5g kernel module from `https://github.com/free5gc/gtp5g.git` \
Build free5c from source by following tutorial at `https://free5gc.org/guide/3-install-free5gc/` 

After building \
Replace `config/upfcfg.yaml` with the provided `upfcfg.yaml` and \
change the IP address in line 6, 7, 16 to the IP address of server B

Running free5gc UPF
`sudo ./bin/upf`

## HiP4-UPF Data plane
We provided the UPF dataplane code used for both section 6.2 and 6.3 in `reproduce/use_this_upf`
You need to use `reproduce/use_this_upf` P4 code here, it supports 150k numbers of simple N6/N3 UEs. Because we only use simple UEs in the evaluation code. \
After building both south and north controller make sure the south controller uses P4 program from `reproduce/use_this_upf`. 

## Reproducing result
### HiP4-UPF Control plane
For control plane you can test UPF throughput and latency using rust project located in `controller/north_controller/demo/mock-smf-comprehensive`. 
- To test Handover throughput, rename `main_handover_throughput.rs` to `main.rs` and change line 643 and 644's IP address to that of P4 switch and your local machine.
- To test Handover latency, rename `main_handover_latency.rs` to `main.rs` and change line 651 and 652's IP address to that of P4 switch and your local machine.
- To test Registration/De-registration throughput, rename `main_reg_dereg_throughput.rs` to `main.rs` and change line 635 and 636's IP address to that of P4 switch and your local machine.

You can change the IP address *and the `upf_name`* in the `main.rs` file to test the performance of open5gs/free5gc/hip4-upf and obtain the result in Table 4.

Results will be saved as a bunch of csv files, 

# Reproducing result in Figure 12
The dotted green line for HiP4 UPF can be reproduced using the provided controller code

The solid grey line for single thread version can be reproduced using the controller and P4 code in `reproduce/section 6.2/single_thread`. The build process is the same. After building the result can be reproduced with `main_handover_latency.rs`.

