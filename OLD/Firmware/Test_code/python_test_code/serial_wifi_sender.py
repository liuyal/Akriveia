import time, sys, os, threading
import serial, io
import socket

com = serial.Serial('COM4', 9600)
msg1 = b""
UDP_IP = "192.168.1.104"
UDP_PORT = 15400

while True:
    msg1 = com.readline(com.inWaiting())
    print(msg1)

    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.sendto(msg1, (UDP_IP, UDP_PORT))

    time.sleep(1)
