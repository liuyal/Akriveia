import socket
import time, sys, os, threading
import serial, io


UDP_IP = "192.168.1.104"
UDP_PORT = 9999

def send(msg):

    while True:
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.sendto(msg, (UDP_IP, UDP_PORT))
        time.sleep(0.25)


try:
    t1 = threading.Thread(target=send,args=[b"1\n"])
    t2 = threading.Thread(target=send,args=[b"2\n"])
    t3 = threading.Thread(target=send,args=[b"3\n"])
    t1.start()
    t2.start()
    t3.start()

except KeyboardInterrupt:
    t1.join()
    t2.join()
    t3.join()
