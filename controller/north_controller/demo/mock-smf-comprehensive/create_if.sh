sudo ifconfig enp24s0f0np0 10.201.0.1 netmask 255.255.255.0
sudo arp  -i enp24s0f0np0 -s 10.201.0.2 6c:ec:5a:3b:bd:cd priv