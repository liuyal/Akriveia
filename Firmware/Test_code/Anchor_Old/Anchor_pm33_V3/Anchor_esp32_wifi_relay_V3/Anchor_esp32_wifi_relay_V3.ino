#include <WiFi.h>
#include <WiFiUdp.h>

#define RXD2 16
#define TXD2 17
#define TRIGGER_PIN 32

const byte numChars = 50;
char receivedChars[numChars];
boolean newData = false;

const char* ssid = "akriveia";
const char* password = "";
const char* hostAddress = "10.0.0.3";
const int UdpPort = 9996;
int wifi_timeout = 10 * 1000;

char incomingPacket[255];
String packet;

WiFiUDP Udp;

void setup() {

  Serial.begin(115200);
  Serial2.begin(9600, SERIAL_8N1, RXD2, TXD2);

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
  Udp.beginPacket(hostAddress, UdpPort);
  Udp.printf("[esp_wifi_on]\n");
  Udp.endPacket();

  pinMode(TRIGGER_PIN, OUTPUT);
  digitalWrite(TRIGGER_PIN, HIGH);
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

void udp_send(String msg) {
  if (WiFi.status() == WL_CONNECTED) {
    Udp.beginPacket(hostAddress, UdpPort);
    Udp.printf((msg + "\n").c_str());
    Udp.endPacket();
  }
}

void loop() {

  int packetSize = Udp.parsePacket();
  if (packetSize) {
    Serial.printf("Received %d bytes from %s:%d\n", packetSize, Udp.remoteIP().toString().c_str(), Udp.remotePort());
    int len = Udp.read(incomingPacket, 255);
    if (len > 0) incomingPacket[len] = 0;
    Serial.printf("UDP Packet Contents: %s", incomingPacket);
    if (String(incomingPacket).indexOf("reboot") >= 0) {
      delay(1000);
      digitalWrite(TRIGGER_PIN, LOW);
      delay(3000);
      digitalWrite(TRIGGER_PIN, HIGH);
      delay(3000);
      udp_send(String("[reboot_ack]"));
      ESP.restart();
    }
    Serial2.println("<" + String(incomingPacket) + '>');
  }

  recvWithStartEndMarkers();
  if (newData == true) {
    Serial.println(String(receivedChars));
    udp_send((String(receivedChars)));
  }
  newData = false;
}
