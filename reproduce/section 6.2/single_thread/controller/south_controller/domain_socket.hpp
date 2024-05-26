#pragma once

#include <sys/socket.h>
#include <sys/un.h>
#include <sys/types.h>
#include <unistd.h>
#include <cstring>
#include <stdexcept>
#include <chrono>

class UnixDomainSocket {
public:
    UnixDomainSocket(const std::string& send_endpoint, std::string const& recv_endpoint) {
        // Create a socket
        sockfd_recv = socket(AF_UNIX, SOCK_DGRAM, 0);
        if (sockfd_recv < 0) {
            throw std::runtime_error("Socket creation failed");
        }

        // Set up the socket address structure
        memset(&addr_recv, 0, sizeof(addr_recv));
        addr_recv.sun_family = AF_UNIX;
        strcpy(addr_recv.sun_path, recv_endpoint.c_str());

        if (bind(sockfd_recv, (struct sockaddr *) &addr_recv, sizeof(addr_recv)) < 0) { 
            std::string error_msg = "Socket creation failed bind: ";
            error_msg += std::strerror(errno);  // Append the error message
            throw std::runtime_error(error_msg);
        }
        // if (listen(sockfd, 5) < 0) { 
        //     std::string error_msg = "Socket creation failed listen: ";
        //     error_msg += std::strerror(errno);  // Append the error message
        //     throw std::runtime_error(error_msg);
        // }

        // if(connect(sockfd, (sockaddr*)&addr, sizeof(addr)) == -1) {
        //     std::string error_msg = "Socket creation failed connect: ";
        //     error_msg += std::strerror(errno);  // Append the error message
        //     throw std::runtime_error(error_msg);
        // }

        int buffer_size = 16 * 1024 * 1024;  // 16 MB, for example
        if (setsockopt(sockfd_recv, SOL_SOCKET, SO_RCVBUF, &buffer_size, sizeof(buffer_size)) < 0) {
            perror("setting receive buffer size failed");
            close(sockfd_recv);
            exit(EXIT_FAILURE);
        }
        if (setsockopt(sockfd_recv, SOL_SOCKET, SO_SNDBUF, &buffer_size, sizeof(buffer_size)) < 0) {
            perror("setting send buffer size failed");
            close(sockfd_recv);
            exit(EXIT_FAILURE);
        }


        // send
        sockfd_send = socket(AF_UNIX, SOCK_DGRAM, 0);
        if (sockfd_send < 0) {
            throw std::runtime_error("Socket creation failed");
        }
        memset(&addr_send, 0, sizeof(addr_send));
        addr_send.sun_family = AF_UNIX;
        strcpy(addr_send.sun_path, send_endpoint.c_str());

        
        connected = false;

    }

    void connect_to_rust() {
        if (connected) {
            return;
        }
        size_t data_len = strlen(addr_send.sun_path) + sizeof(addr_send.sun_family);
        if( connect(sockfd_send, (struct sockaddr*)&addr_send, data_len) == -1 ) {
            std::string error_msg = "Send socket creation failed connect: ";
            error_msg += std::strerror(errno);  // Append the error message
            throw std::runtime_error(error_msg);
        }
        connected = true;
    }

    ~UnixDomainSocket() {
        unlink(addr_recv.sun_path);
        close(sockfd_recv);
        close(sockfd_send);
    }

    void send(const std::byte* data, size_t size) {
        connect_to_rust();
        size_t sent = 0;
        while (sent < size) {
            ssize_t result = ::sendto(sockfd_send, reinterpret_cast<const char*>(data) + sent, 
                                      size - sent, 0, 
                                      reinterpret_cast<sockaddr*>(&addr_send), sizeof(addr_send));
            if (result < 0) {
                std::string error_msg = "Send failed: ";
                error_msg += std::strerror(errno);  // Append the error message
                throw std::runtime_error(error_msg);
            }
            sent += result;
        }
    }

    ssize_t receive(std::byte* dst, std::size_t dst_buffer_size, uint64_t timeout_us) {
        fd_set readfds;
        FD_ZERO(&readfds);
        FD_SET(sockfd_recv, &readfds);

        timeval tv;
        tv.tv_sec = timeout_us / 1000000;
        tv.tv_usec = timeout_us % 1000000;

        int result = select(sockfd_recv + 1, &readfds, nullptr, nullptr, &tv);
        if (result == 0) {
            return -2; // Timed out
        }

        if (result < 0) {
            throw std::runtime_error("Error in select");
        }

        socklen_t addr_len = sizeof(addr_recv);
        ssize_t recv_len = recvfrom(sockfd_recv, reinterpret_cast<char*>(dst), dst_buffer_size, 0, 
                                    reinterpret_cast<sockaddr*>(&addr_recv), &addr_len);

        if (recv_len < 0) {
            throw std::runtime_error("Receive failed");
        }

        if (static_cast<size_t>(recv_len) > dst_buffer_size) {
            return -1; // Data exceeds length of buffer
        }

        return recv_len; // Data received successfully
    }

private:
    int sockfd_recv;
    int sockfd_send;
    sockaddr_un addr_recv;
    sockaddr_un addr_send;
    bool connected;
};
