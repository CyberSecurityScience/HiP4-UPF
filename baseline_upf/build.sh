#!/bin/bash
NAME=upf
rm -f bf_drivers.log*
rm -f *.log
rm -rf $SDE_INSTALL/$NAME.tofino/
rm -rf $NAME.tofino/
bf-p4c $NAME.p4 --create-graphs --display-power-budget --log-hashes -g
cp -R $NAME.tofino $SDE_INSTALL/