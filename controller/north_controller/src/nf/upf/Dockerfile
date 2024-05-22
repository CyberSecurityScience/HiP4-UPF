FROM ubuntu:18.04

RUN apt update
RUN apt install -y iproute2
RUN apt install -y tar
RUN apt install -y iptables

# These commands copy your files into the specified directory in the image
# and set that as the working location
COPY utils/gtp_gateway /app/gtp_gateway
COPY utils/internet_gateway /app/internet_gateway
COPY utils/startup.sh /app/startup.sh
COPY utils/tcpdump /app/tcpdump
COPY p4 /app/p4
COPY target/x86_64-unknown-linux-musl/release/upf /app/upf
COPY cgnat/nat /app/nat
#COPY simple_switch_grpc/ /
WORKDIR /app

EXPOSE 2152/udp
EXPOSE 8805/udp
EXPOSE 80
EXPOSE 443

ENV LD_LIBRARY_PATH /usr/local/lib:$LD_LIBRARY_PATH

RUN chmod +x startup.sh

# This command runs your application, comment out this line to compile only
ENTRYPOINT ["./startup.sh"]
