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

#define BUF_SIZ 3000

char ifNameRaw[IFNAMSIZ];
char ifNameTun[IFNAMSIZ];

int raw_fd, tun_fd;
int raw_ifidx;

int tun_open() {
  struct ifreq ifr;
  int fd, err;

  if ((fd = open("/dev/net/tun", O_RDWR)) == -1) {
    perror("open /dev/net/tun");
    exit(1);
  }
  memset( & ifr, 0, sizeof(ifr));
  ifr.ifr_flags = IFF_TUN;
  strncpy(ifr.ifr_name, ifNameTun, IFNAMSIZ); // ifNameTun = "tun0" or "tun1", etc 

  /* ioctl will use ifr.if_name as the name of TUN 
   * interface to open: "tun0", etc. */
  if ((err = ioctl(fd, TUNSETIFF, (void * ) & ifr)) == -1) {
    perror("ioctl TUNSETIFF");
    close(fd);
    exit(1);
  }

  /* After the ioctl call the fd is "connected" to tun device specified
   * by ifNameTun ("tun0", "tun1", etc)*/

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
  for (;;) {
    numbytes = recvfrom(raw_fd, buf, BUF_SIZ, 0, NULL, NULL);
    //printf("listener[RAW]: got packet %lu bytes\n", numbytes);

    /* Print packet */
    // printf("\tData:");
    // for (int i=14; i<numbytes; i++) printf("%02x:", buf[i]);
    // printf("\n");

    if (write(tun_fd, buf + 10, numbytes - 10) < 0)
      perror("write");
  }
  return NULL;
}

void * tun_recv_thread(void * unused) {
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

   struct sockaddr_ll socket_address;
   socket_address.sll_ifindex = raw_ifidx;
   /* Address length*/
   socket_address.sll_halen = ETH_ALEN;
   /* Destination MAC */
   socket_address.sll_addr[0] = 0x34;
   socket_address.sll_addr[1] = 0x34;
   socket_address.sll_addr[2] = 0x34;
   socket_address.sll_addr[3] = 0x34;
   socket_address.sll_addr[4] = 0x34;
   socket_address.sll_addr[5] = 0x34;

  printf("listener[TUN]: Waiting to recvfrom on %s...\n", ifNameTun);
  for (;;) {
    numbytes = read(tun_fd, buf + 10, BUF_SIZ - 14); //recvfrom(tun_fd, buf+ sizeof(struct ether_header), BUF_SIZ- sizeof(struct ether_header), 0, NULL, NULL);
    //printf("listener[TUN]: got packet %lu bytes\n", numbytes);
    uint8_t * ip_packet = buf + 14;
    uint8_t ipver = ( * ip_packet) >> 4;
    if (ipver & 0x60) {
      eh -> ether_type = htons(ETH_P_IPV6);
    } else {
      eh -> ether_type = htons(ETH_P_IP);
    }

    /* Print packet */
    // printf("\tData:");
    // for (int i=0; i<numbytes; i++) printf("%02x:", buf[i]);
    // printf("\n");
    // for (int i=14; i<numbytes; i++) printf("%02x:", buf[i]);
    // printf("\n");

    sendto(raw_fd, buf, numbytes + 14, 0, (struct sockaddr * ) & socket_address, sizeof(struct sockaddr_ll));
    //if (write(raw_fd, buf, numbytes + 14))
    //  perror("write");
  }
  return NULL;
}

int main(int argc, char * argv[]) {
  /* Get interface name */
  if (argc > 2) {
    strcpy(ifNameRaw, argv[1]);
    strcpy(ifNameTun, argv[2]);
  } else {
    printf("Usage: %s <raw interface> <tun interface>\n", argv[0]);
    return 1;
  }

  raw_fd = raw_open();
  tun_fd = tun_open();

  pthread_t threads[2];

  int err = pthread_create( & (threads[0]), NULL, raw_recv_thread, NULL);
  if (err != 0) {
    printf("Can't create thread :[%s]\n", strerror(err));
    exit(1);
  } else
    printf("Raw socket thread created successfully\n");

  err = pthread_create( & (threads[1]), NULL, tun_recv_thread, NULL);
  if (err != 0) {
    printf("Can't create thread :[%s]\n", strerror(err));
    exit(1);
  } else
    printf("Tun socket thread created successfully\n");

  pthread_join(threads[0], NULL);
  pthread_join(threads[1], NULL);

  close(raw_fd);
  close(tun_fd);

  return 0;
}
