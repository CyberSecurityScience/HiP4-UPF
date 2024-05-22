#include <unistd.h>
#include <arpa/inet.h>
#include <linux/if_packet.h>
#include <linux/ip.h>
#include <linux/udp.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <sys/ioctl.h>
#include <sys/socket.h>
#include <net/if.h>
#include <netinet/ether.h>
#include <sys/types.h>
#include <linux/if.h>
#include <linux/if_tun.h>
#include <fcntl.h>
#include <pthread.h>

unsigned short gtp_csum(unsigned short * ptr, int nbytes) {
  register long sum;
  unsigned short oddbyte;
  register short answer;

  sum = 0;
  while (nbytes > 1) {
    sum += * ptr++;
    nbytes -= 2;
  }
  if (nbytes == 1) {
    oddbyte = 0;
    *((u_char * ) & oddbyte) = * (u_char * ) ptr;
    sum += oddbyte;
  }

  sum = (sum >> 16) + (sum & 0xffff);
  sum = sum + (sum >> 16);
  answer = (short) ~sum;

  return (answer);
}

#define ETHER_TYPE 0x0800

#define DEFAULT_IF "eth0"
#define BUF_SIZ 3000

char ifNameRaw[IFNAMSIZ];

int raw_fd, udp_fd;
int raw_ifidx;

int udp_open() {
  int fd;
  struct sockaddr_in server_addr;

  // Create UDP socket:
  fd = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);

  if (fd < 0) {
    printf("[UDP] Error while creating socket\n");
    return -1;
  }
  printf("[UDP] Socket created successfully\n");

  // Set port and IP:
  server_addr.sin_family = AF_INET;
  server_addr.sin_port = htons(2152);
  server_addr.sin_addr.s_addr = inet_addr("0.0.0.0");

  int yes = 1;
  if (setsockopt(fd, IPPROTO_IP, IP_PKTINFO, & yes, sizeof(yes)) < 0) {
    perror("IP_PKTINFO");
  }

  // Bind to the set port and IP:
  if (bind(fd, (struct sockaddr * ) & server_addr, sizeof(server_addr)) < 0) {
    printf("[UDP] Couldn't bind to the port\n");
    return -1;
  }

  return fd;
}

int raw_open() {
  int sockfd;
  struct ifreq if_ip; /* get ip addr */

  memset( & if_ip, 0, sizeof(struct ifreq));

  /* Open PF_PACKET socket, listening for EtherType ETHER_TYPE */
  if ((sockfd = socket(PF_PACKET, SOCK_RAW, htons(ETH_P_ALL))) == -1) {
    perror("listener: socket");
    return -1;
  }
  struct ifreq ifr;
  memset( & ifr, 0, sizeof(struct ifreq));
  memcpy(ifr.ifr_name, ifNameRaw, strlen(ifNameRaw));
  if (ioctl(sockfd, SIOCGIFINDEX, & ifr) == -1) {
    perror("ioctl");
    exit(1);
  }
  raw_ifidx = ifr.ifr_ifindex;

  struct sockaddr_ll addr = {
    0
  };
  addr.sll_family = AF_PACKET;
  addr.sll_ifindex = raw_ifidx;
  addr.sll_protocol = htons(ETH_P_ALL);
  if (bind(sockfd, (struct sockaddr * ) & addr, sizeof(addr)) == -1) {
    perror("bind");
    exit(1);
  }

  return sockfd;
}

void * raw_recv_thread(void * unused) {
  (void) unused;
  uint8_t buf[BUF_SIZ];
  ssize_t numbytes;

  printf("listener[RAW]: Waiting to recvfrom on %s...\n", ifNameRaw);
  struct sockaddr_in target_addr;
  int slen = sizeof(target_addr);
  memset( & target_addr, 0, slen);
  for (;;) {
    numbytes = recvfrom(raw_fd, buf, BUF_SIZ, 0, NULL, NULL);
    //printf("listener[RAW]: got packet %lu bytes\n", numbytes);
    struct iphdr * iph = (struct iphdr * )(buf + 14);
    target_addr.sin_addr.s_addr = iph -> daddr;
    target_addr.sin_family = AF_INET;
    target_addr.sin_port = htons(2152);

    /* Print packet */
    // printf("\tData:");
    // for (int i=14; i<numbytes; i++) printf("%02x:", buf[i]);
    // printf("\n");

    if (sendto(udp_fd, buf + 14 + 20 + 8, numbytes - (14 + 20 + 8),
        MSG_CONFIRM, (const struct sockaddr * ) & target_addr,
        slen) < 0) perror("write");
    // if (write(udp_fd, buf +10, numbytes-10)<0)
    // 	perror("write");
  }
  return NULL;
}

void * udp_recv_thread(void * unused) {
  (void) unused;
  uint8_t buf[BUF_SIZ];
  ssize_t numbytes;

  struct ether_header * eh = (struct ether_header * ) buf;
  eh -> ether_shost[0] = 0x34;
  eh -> ether_shost[1] = 0x34;
  eh -> ether_shost[2] = 0x33;
  eh -> ether_shost[3] = 0x33;
  eh -> ether_shost[4] = 0x33;
  eh -> ether_shost[5] = 0x33;
  eh -> ether_dhost[0] = 0x34;
  eh -> ether_dhost[1] = 0x34;
  eh -> ether_dhost[2] = 0x34;
  eh -> ether_dhost[3] = 0x34;
  eh -> ether_dhost[4] = 0x34;
  eh -> ether_dhost[5] = 0x34;
  eh -> ether_type = htons(ETHER_TYPE);

  struct iphdr * iph = (struct iphdr * )(buf + 14);
  iph -> ihl = 5;
  iph -> version = 4;
  iph -> tos = 0;
  //iph->tot_len = htons(sizeof (struct iphdr) + sizeof (struct udphdr) + fullSize);
  iph -> id = random() & 0xffff; //Id of this packet
  iph -> frag_off = 0;
  iph -> ttl = 64;
  iph -> protocol = IPPROTO_UDP;
  iph -> check = 0; //Set to 0 before calculating checksum
  //iph->saddr = inet_addr ( gtp_src_ip );	//Spoof the source ip address
  //iph->daddr = to.sin_addr.s_addr;
  //iph->check = gtp_csum ((unsigned short *) iph, sizeof (struct iphdr) + sizeof (struct udphdr) + fullSize);

  struct udphdr * udphdr = (struct udphdr * )(buf + 20 + 14);
  udphdr -> dest = htons(2152);
  udphdr -> source = htons(2152);
  //udphdr->len

  // the control data is dumped here
  char cmbuf[0x100];
  // the remote/source sockaddr is put here
  struct sockaddr_in peeraddr;
  struct iovec iov = {
    .iov_base = (void * )(buf + 20 + 14 + 8),
    .iov_len = BUF_SIZ - (20 + 14 + 8)
  };
  // if you want access to the data you need to init the msg_iovec fields
  struct msghdr mh = {
    .msg_iov = & iov,
    .msg_iovlen = 1,
    .msg_name = & peeraddr,
    .msg_namelen = sizeof(peeraddr),
    .msg_control = cmbuf,
    .msg_controllen = sizeof(cmbuf),
  };

  printf("listener[UDP]: Waiting to recvfrom UDP port 2152...\n");
  for (;;) {
    // numbytes = recvfrom(udp_fd, buf+20+14+8, BUF_SIZ-(20+14+8), 0,
    //      (struct sockaddr*)&server_addr, &sl);
    numbytes = recvmsg(udp_fd, & mh, 0);
    if (numbytes < 0) {
      perror("recvmsg");
    }
    //numbytes = read(udp_fd,buf+10,BUF_SIZ- 14);//recvfrom(tun_fd, buf+ sizeof(struct ether_header), BUF_SIZ- sizeof(struct ether_header), 0, NULL, NULL);
    //printf("listener[UDP]: got packet %lu bytes\n", numbytes);
    iph -> saddr = peeraddr.sin_addr.s_addr;
    for ( // iterate through all the control headers
      struct cmsghdr * cmsg = CMSG_FIRSTHDR( & mh); cmsg != NULL; cmsg = CMSG_NXTHDR( & mh, cmsg)) {
      // ignore the control headers that don't match what we want
      if (cmsg -> cmsg_level != IPPROTO_IP ||
        cmsg -> cmsg_type != IP_PKTINFO) {
        continue;
      }
      struct in_pktinfo * pi = (struct in_pktinfo * ) CMSG_DATA(cmsg);
      // at this point, peeraddr is the source sockaddr
      // pi->ipi_spec_dst is the destination in_addr
      // pi->ipi_addr is the receiving interface in_addr
      iph -> daddr = pi -> ipi_spec_dst.s_addr;
      break;
    }
    udphdr -> len = htons(numbytes + 8);
    udphdr -> check = 0;
    iph -> tot_len = htons(20 + 8 + numbytes);
    iph -> check = 0;
    iph -> check = gtp_csum((unsigned short * ) iph, sizeof(struct iphdr));

    /* Print packet */
    // printf("\tData:");
    // for (int i=0; i<numbytes; i++) printf("%02x:", buf[i]);
    // printf("\n");
    // for (int i=14; i<numbytes; i++) printf("%02x:", buf[i]);
    // printf("\n");

    //sendto(udp_fd, buf, numbytes +14+20+8, 0, (struct sockaddr*)&socket_address, sizeof(struct sockaddr_ll));
    if (write(raw_fd, buf, numbytes + 20 + 8 + 14) < 0)
      perror("write");
  }
  return NULL;
}

int main(int argc, char * argv[]) {
  /* Get interface name */
  if (argc > 1) {
    strcpy(ifNameRaw, argv[1]);
  } else {
    printf("Usage: %s <raw interface>\n", argv[0]);
    return 1;
  }

  raw_fd = raw_open();
  udp_fd = udp_open();

  pthread_t threads[2];

  int err = pthread_create( & (threads[0]), NULL, raw_recv_thread, NULL);
  if (err != 0) {
    printf("Can't create thread :[%s]\n", strerror(err));
    exit(1);
  } else
    printf("Raw socket thread created successfully\n");

  err = pthread_create( & (threads[1]), NULL, udp_recv_thread, NULL);
  if (err != 0) {
    printf("Can't create thread :[%s]\n", strerror(err));
    exit(1);
  } else
    printf("Tun socket thread created successfully\n");

  pthread_join(threads[0], NULL);
  pthread_join(threads[1], NULL);

  close(raw_fd);
  close(udp_fd);

  return 0;
}
