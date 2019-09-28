#include <WiFi.h>
#include <WiFiUdp.h>

#define RXD2 16
#define TXD2 17

const byte numChars = 50;
char receivedChars[numChars];
boolean newData = false;

const char* ssid = "AP";
const char* password = "";
const char* hostAddress = "192.168.1.104";
const int UdpPort = 15400;
int wifi_timeout = 10 * 1000;

char incomingPacket[255];
bool system_on = false;
String packet;

WiFiUDP Udp;

void setup() {

  Serial.begin(115200);
  Serial2.begin(9600, SERIAL_8N1, RXD2, TXD2);

  WiFi.config(staticIP, subnet, gateway, dns);
  WiFi.begin(ssid, password);
  Serial.println("Connecting to WiFi");
  unsigned long start_wait = millis();
  while (WiFi.status() != WL_CONNECTED && millis() - start_wait <= wifi_timeout) {
    Serial.print(".");
    delay(500);
  }

  Serial.println("Connected! IP address: " + WiFi.localIP().toString());
  Serial.printf("UDP port %d\n", UdpPort);
  Udp.begin(UdpPort);
}

void recvWithStartEndMarkers() {
  static boolean recvInProgress = false;
  static byte ndx = 0;
  char startMarker = '<';
  char endMarker = '>';
  char rc;
  while (Serial2.available() > 0 && newData == false) {
    rc = Serial2.read();
    if (recvInProgress == true) {
      if (rc != endMarker) {
        receivedChars[ndx] = rc;
        ndx++;
        if (ndx >= numChars)  ndx = numChars - 1;
      }
      else {
        receivedChars[ndx] = '\0';
        recvInProgress = false;
        ndx = 0;
        newData = true;
      }
    }
    else if (rc == startMarker) recvInProgress = true;
  }
}


void loop() {

  int packetSize = Udp.parsePacket();
  if (packetSize) {
    Serial.printf("Received %d bytes from %s:%d\n", packetSize, Udp.remoteIP().toString().c_str(), Udp.remotePort());
    int len = Udp.read(incomingPacket, 255);
    if (len > 0) incomingPacket[len] = 0;
    Serial.printf("UDP Packet Contents: %s", incomingPacket);
    Serial2.println("<" + String(incomingPacket) + '>');
  }

  recvWithStartEndMarkers();
  if (newData == true) {
    Serial.println(String(receivedChars));
    if (WiFi.status() == WL_CONNECTED) {
      Udp.beginPacket(hostAddress, UdpPort);
      Udp.printf((String(receivedChars) + "\n").c_str());
      Udp.endPacket();
    }
    newData = false;
  }


}
