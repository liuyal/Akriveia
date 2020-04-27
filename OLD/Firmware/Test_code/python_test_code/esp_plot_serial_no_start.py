import time, sys, os, threading
import serial, io
import matplotlib.pyplot as plt
import matplotlib.animation as animation
from matplotlib import style

size = 10
max = 3.00
min = 0.00

x_1 = 0.00
y_1 = 0.00
x_2 = 0.00
y_2 = max
x_3 = max
y_3 = 0.00

dL1 = 0.00
dL2 = 0.00
dL3 = 0.00

data = []
index = []

def data_maker(a1,a2,a3):
    global data
    global index
    global dL1
    global dL2
    global dL3

    while True:
        msg1 = (a1.readline())
        msg2 = (a2.readline())
        msg3 = (a3.readline())

        try:
            d1 = float(str(msg1).split('|')[-1].replace('\\r\\n\'', ''))
            d2 = float(str(msg2).split('|')[-1].replace('\\r\\n\'', ''))
            d3 = float(str(msg3).split('|')[-1].replace('\\r\\n\'', ''))
        except:
            d1 = dL1
            d2 = dL2
            d3 = dL3


        dL1 = d1
        dL2 = d2
        dL3 = d3

        print(str(d1) + "|" + str(d2) + "|" + str(d3))

        # Trilateration solver
        A = -2.00 * x_1 + 2.00 * x_2
        B = -2.00 * y_1 + 2.00 * y_2
        C = d1 * d1 - d2 * d2 - x_1 * x_1 + x_2 * x_2 - y_1 * y_1 + y_2 * y_2
        D = -2.00 * x_2 + 2.00 * x_3
        E = -2.00 * y_2 + 2.00 * y_3
        F = d2 * d2 - d3 * d3 - x_2 * x_2 + x_3 * x_3 - y_2 * y_2 + y_3 * y_3
        x = (C * E - F * B) / (E * A - B * D)
        y = (C * D - A * F) / (B * D - A * E)

        print(str(x) + '|' + str(y))
        try:
            index = []; index.append(abs(x))
            data = []; data.append(abs(y))
        except: None

        # time.sleep(1)


def plotter():
    def animate(i):
        x = index
        y = data

        ax1.clear()
        ax1.set_ylim(bottom=0, top=5)
        ax1.set_xlim(left=0, right=5)
        ax1.scatter(x, y)

        circle1 = plt.Circle((0, 0), dL1, fc='r', ec='r', lw=2, alpha=0.5)
        circle2 = plt.Circle((0, 3), dL2, fc='b', ec='b', lw=2, alpha=0.5)
        circle3 = plt.Circle((3, 0), dL3, fc='g', ec='g', lw=2, alpha=0.5)

        ax1.add_artist(circle1)
        ax1.add_artist(circle2)
        ax1.add_artist(circle3)

    style.use('seaborn-whitegrid')
    fig = plt.figure(num=1, figsize=(10,4), dpi=100)
    ax1 = fig.add_subplot(1, 1, 1)
    ani = animation.FuncAnimation(fig, animate, interval=1000)
    plt.grid(True)
    plt.show()

if __name__ == "__main__":

    a1 = serial.Serial('COM7', 115200)
    a2 = serial.Serial('COM4', 115200)
    a3 = serial.Serial('COM6', 115200)
    msg1 = b""
    msg2 = b""
    msg3 = b""
    while True:
        if b"|" in msg1 :
            break
        else:
            msg1 = a1.read(a1.inWaiting())
            msg2 = a2.read(a2.inWaiting())
            msg3 = a3.read(a3.inWaiting())
            print(msg1)
            print(msg2)
            print(msg3)

    print("\n-Start-\n")
    try:
        t1 = threading.Thread(target=data_maker,args=[a1,a2,a3])
        t2 = threading.Thread(target=plotter)
        t1.start()
        time.sleep(2)
        t2.start()
    except KeyboardInterrupt:
        t1.join()
        t2.join()


