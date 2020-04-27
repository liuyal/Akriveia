import serial, os, sys, time, io

measure_power = -76.00
N = 2.00

x_1 = 0.00
y_1 = 0.00

x_2 = 3.00
y_2 = 0.00

x_3 = 0.00
y_3 = 3.00

a1 = serial.Serial('COM4', 115200)
a2 = serial.Serial('COM3', 115200)
a3 = serial.Serial('COM10', 115200)

msg1 = b""
msg2 = b""
msg3 = b""

while True:
    if b"ack" in msg1 and b"ack" in msg2 and b"ack" in msg3:
        break
    else:
        a1.write('start\n'.encode())
        a2.write('start\n'.encode())
        a3.write('start\n'.encode())
        a1.flush()
        a2.flush()
        a3.flush()
        time.sleep(5)
        msg1 = a1.read(a1.inWaiting())
        msg2 = a2.read(a2.inWaiting())
        msg3 = a3.read(a3.inWaiting())
        print(msg1)
        print(msg2)
        print(msg3)

while True:
    msg1 = (a1.readline())
    msg2 = (a2.readline())
    msg3 = (a3.readline())

    n1 = float(str(msg1).split('|')[-1].replace('\\r\\n\'', ''))
    n2 = float(str(msg2).split('|')[-1].replace('\\r\\n\'', ''))
    n3 = float(str(msg3).split('|')[-1].replace('\\r\\n\'', ''))

    d1 = 10.00**((measure_power-(n1))/(10.00*N))
    d2 = 10.00**((measure_power-(n2))/(10.00*N))
    d3 = 10.00**((measure_power-(n3))/(10.00*N))

    # Trilateration solver
    A = -2.00*x_1 + 2.00*x_2
    B = -2.00*y_1 + 2.00*y_2
    C = d1*d1 - d2*d2 - x_1*x_1 + x_2*x_2 - y_1*y_1 + y_2*y_2
    D = -2.00*x_2 + 2.00*x_3
    E = -2.00*y_2 + 2.00*y_3
    F = d2*d2 - d3*d3 - x_2*x_2 + x_3*x_3 - y_2*y_2 + y_3*y_3

    x = (C*E - F*B) / (E*A - B*D)
    y = (C*D - A*F) / (B*D - A*E)

    print(str(x) + '|' + str(y))
    time.sleep(1)
