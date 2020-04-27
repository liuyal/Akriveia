import socket,time

UDP_IP = ["192.168.1.69","192.168.1.70","192.168.1.75"]
UDP_PORT = 9996
MESSAGE = b"end\n"

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.sendto(MESSAGE, (UDP_IP[0], UDP_PORT))
time.sleep(0.25)
sock.sendto(MESSAGE, (UDP_IP[1], UDP_PORT))
time.sleep(0.25)
sock.sendto(MESSAGE, (UDP_IP[2], UDP_PORT))