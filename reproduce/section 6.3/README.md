# Reproducing result in Figure 11
We provide a simplified traffic generator the reproduce the result, which is located at `reproduce/section 6.3/reporting-delay`.

Build it with `cargo build --release`

## Setting up interface
We use the same hardware config used in section 6.2

Assuming a 100G cable connecting server A and Tofino switch. The interface for that port on server A is `enp24s0f0np0` \
First, enable the port with `sudo ifconfig enp24s0f0np0 10.201.0.1/24 up` 
\
Then setup ARP with `sudo arp  -i enp24s0f0np0 -s 10.201.0.2 6c:ec:5a:3b:bd:cd priv`
Make sure the IP and MAC address is untouched as they are hardcoded into the eval code for now.

## Running the experiment
After running the south controller with `./upf_driver --install-dir $SDE_INSTALL --conf-file $SDE_INSTALL/upf.tofino/upf.conf --ucli` \
You need to config the ports. a command line interface will show up after the south controller has started, run the following to show up ports:
```
pm
port-add -/- 100G RS
port-enb -/-
an-set -/- 1
show
```

Change the IP address of `reproduce/section 6.3/reporting-delay/src/main.rs` in line 316 and 317.

By default the south controller code uses selective pulling, you can run the `reporting-delay` program and measure reporting delay latency w/ selective pulling (solid green line Figure 11) by running
```
export RUST_LOG=info
./target/release/mock-smf --ue 150000 --name reporting-delay-with-selective-pulling-exp1
```
Result will be saved tp `result-reporting-delay-with-selective-pulling-exp1-150000.csv`


### Disabling selective pulling
To disable selective pulling, change the pulling batch size in the south controller to 160000 by changing line 40 in `controller/south_controller/accounting_tables.hpp` and rebuild the south controller.

After restarting both south and north controller, you can test the reporting delay without selective pulling (dotted orange line in Figure 11) with `./target/release/mock-smf --ue 150000 --name reporting-delay-without-selective-pulling-exp1`

After both experiments are done, run `python3 eval.py .`\
It will show the mean, stddev and 99 percentile reporting delay of both experiments, enough to show the trend in Figure 11.

