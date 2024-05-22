#!/bin/bash

ip a

POSITIONAL=()
while [[ $# -gt 0 ]]; do
  key="$1"

  case $key in
    --dump-pcap)
      DUMP_PCAP="$2"
      shift # past argument
      shift # past value
      ;;
    --dump-pcap-up)
      DUMP_PCAP_UP="$2"
      shift # past argument
      shift # past value
      ;;
    --cgnat-ip)
      CGNAT_IP="$2"
      shift # past argument
      shift # past value
      ;;
    -m|--mode)
      MODE="$2"
      shift # past argument
      shift # past value
      ;;
    --ip-range)
      IP_RANGE="$2"
      shift # past argument
      shift # past value
      ;;
    *)    # unknown option
      POSITIONAL+=("$1") # save it in an array for later
      shift # past argument
      ;;
  esac
done

set -- "${POSITIONAL[@]}"

if [ ! -z ${CGNAT_IP+x} ]; then
    USE_CGNAT=true
    IP_RANGE=${CGNAT_IP}/16
fi

if [ -z ${MODE+x} ]; then
    MODE=docker
fi

if [ "$USE_CGNAT" == true ]; then
    echo "Using CGNAT public IP ${CGNAT_IP}"
fi

if [ -z ${IP_RANGE+x} ]; then
    echo "Please specify UE IP range"
    exit
fi

if [ "$MODE" == "k8s" ]; then
    echo "Running in k8s mode"
    host=$(ip a|grep 10|grep -oE '((1?[0-9][0-9]?|2[0-4][0-9]|25[0-5])\.){3}(1?[0-9][0-9]?|2[0-4][0-9]|25[0-5])'|head -n 1)
    echo "Using host IP ${host}"
elif [ "$MODE" == "docker" ]; then
    echo "Running in docker mode"
    host=$(ip a|grep 172|grep -oE '((1?[0-9][0-9]?|2[0-4][0-9]|25[0-5])\.){3}(1?[0-9][0-9]?|2[0-4][0-9]|25[0-5])'|head -n 1)
    echo "Using host IP ${host}"
else
    echo "Error: Unknown UPF mode"
    exit
fi

mkdir -p /dev/net
mknod /dev/net/tun c 10 200
chmod 600 /dev/net/tun

ip tuntap add tun0 mode tun
ip addr add $IP_RANGE dev tun0
ip link set dev tun0 up

echo "Final IP range is $IP_RANGE"
iptables -t nat -A POSTROUTING -s $IP_RANGE -o eth0 -j SNAT --to-source $host

ip link add dummy0 type dummy
ip link set dummy0 mtu 2300
ip link add dummy1 type dummy
ip link set dev dummy0 up
ip link set dev dummy1 up

if [ "$USE_CGNAT" == true ]; then
    ip link add dummy_nat type dummy
    ip link set dev dummy_nat up
    ./gtp_gateway dummy0 &
    ./internet_gateway dummy1 tun0 &
    simple_switch_grpc --no-p4 -i 0@dummy_nat -i 1@dummy1 -- --grpc-server-addr 127.0.0.1:50001 --cpu-port 255 &
    simple_switch_grpc --no-p4 -i 0@dummy0 -i 1@dummy_nat -- --grpc-server-addr 127.0.0.1:8080 --cpu-port 255 &
    sleep 1
    ./nat --pubip $CGNAT_IP &
else
    ./gtp_gateway dummy0 &
    ./internet_gateway dummy1 tun0 &
    simple_switch_grpc --no-p4 -i 0@dummy0 -i 1@dummy1 -- --grpc-server-addr 127.0.0.1:8080 --cpu-port 255 &
fi

if [ ! -z ${DUMP_PCAP+x} ]; then
    echo "Pcap dump on any interface enabled"
    mkdir -p $DUMP_PCAP
    touch $DUMP_PCAP/any.pcap
    chmod 777 $DUMP_PCAP_UP/any.pcap
    ./tcpdump -U -q -n -K -i any -w $DUMP_PCAP/any.pcap &
fi

if [ ! -z ${DUMP_PCAP_UP+x} ]; then
    echo "Pcap dump for UP traffic enabled"
    mkdir -p $DUMP_PCAP_UP
    touch $DUMP_PCAP/gtpu.pcap
    touch $DUMP_PCAP/internet.pcap
    chmod 777 $DUMP_PCAP_UP/gtpu.pcap
    chmod 777 $DUMP_PCAP_UP/internet.pcap
    ./tcpdump -U -q -n -K -i dummy0 -w $DUMP_PCAP_UP/gtpu.pcap &
    ./tcpdump -U -q -n -K -i tun0 -w $DUMP_PCAP_UP/internet.pcap &
fi



sleep 1
exec ./upf $@ --ip $host
