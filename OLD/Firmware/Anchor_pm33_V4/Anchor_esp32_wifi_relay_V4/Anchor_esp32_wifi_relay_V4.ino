#include <EEPROM.h>
#include <WiFi.h>
#include <WiFiUdp.h>

#define RXD2 16
#define TXD2 17
#define TRIGGER_PIN 32

const byte numChars = 100;
char receivedChars[numChars];
boolean newData = false;
String EUI = "AA:BB:CC:DD:EE:FF:00:00";

char* ssid = "akriveia";
char* password = "";
IPAddress hostIP(192, 168, 1, 104);
int UdpPort = 9996;
int wifi_timeout = 10 * 1000;

char incomingPacket[255];
String packet;

WiFiUDP Udp;

void setup() {
  Serial.begin(115200);
  Serial2.begin(9600, SERIAL_8N1, RXD2, TXD2);
  pinMode(TRIGGER_PIN, OUTPUT);
  digitalWrite(TRIGGER_PIN, HIGH);
  EEPROM.begin(64);
  IPAddress IP(EEPROM.read(0) , EEPROM.read(1) , EEPROM.read(2) , EEPROM.read(3));
  hostIP = IP;
  WiFi.begin(ssid, password);
  Serial.print("Connecting to WiFi");
  unsigned long start_wait = millis();
  while (WiFi.status() != WL_CONNECTED && millis() - start_wait <= wifi_timeout) {
    Serial.print(".");
    delay(500);
  }
  Serial.println("\nConnected to: " + String(ssid));
  Serial.println("Gateway IP: " + WiFi.gatewayIP().toString());
  Serial.println("Local IP: " + WiFi.localIP().toString());
  Serial.printf("UDP port: %d\n", UdpPort);
  Serial.println("Host IP: " + hostIP.toString());
  delay(3000);
  Serial2.println("<get_eui>");
  Udp.begin(UdpPort);
}

IPAddress str_to_ip(String msg) {
  int Parts[4] = {0, 0, 0, 0};
  int Part = 0;
  for ( int i = 0; i < msg.length(); i++ ) {
    char c = msg[i];
    if ( c == '.' ) {
      Part++; continue;
    }
    Parts[Part] *= 10;
    Parts[Part] += c - '0';
  }
  IPAddress IP(Parts[0], Parts[1], Parts[2], Parts[3]);
  EEPROM.write(0, Parts[0]); EEPROM.write(1, Parts[1]);
  EEPROM.write(2, Parts[2]); EEPROM.write(3, Parts[3]);
  EEPROM.commit();
  return IP;
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
        receivedChars[ndx] = rc; ndx++;
        if (ndx >= numChars) ndx = numChars - 1;
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
    Udp.beginPacket(hostIP, UdpPort);
    Udp.printf((msg + "\n").c_str());
    Udp.endPacket();
  }
}

void loop() {
  int packetSize = Udp.parsePacket();
  if (packetSize) {
    int len = Udp.read(incomingPacket, 255);
    if (len > 0) incomingPacket[len] = 0;
    Serial.printf("Received %d bytes from %s:%d\n", packetSize, Udp.remoteIP().toString().c_str(), Udp.remotePort());
    Serial.printf("UDP Packet Contents: %s", incomingPacket);
    if (String(incomingPacket).indexOf("reboot") >= 0) {
      digitalWrite(TRIGGER_PIN, LOW);
      delay(3000);
      digitalWrite(TRIGGER_PIN, HIGH);
      udp_send("[" + EUI + "|reboot_ack]");
      delay(1000);
      ESP.restart();
    }
    else if (String(incomingPacket).indexOf("set_ip") >= 0) {
      int start_index = String(incomingPacket).indexOf("|") + 1;
      int end_index = String(incomingPacket).indexOf("]");
      String msg = String(incomingPacket).substring(start_index, end_index);
      IPAddress IP = str_to_ip(msg);
      hostIP = IP;
      Serial.println("Host IP Set: " + hostIP.toString());
      udp_send("[" + EUI + "|set_ip_ack]");
    }
    Serial2.println("<" + String(incomingPacket) + ">");
  }

  recvWithStartEndMarkers();
  if (newData == true) {
    Serial.println(String(receivedChars));
    if (String(receivedChars).indexOf("eui_ack") >= 0) {
      EUI = String(receivedChars).substring(0, 23);
      udp_send("[" + EUI + "|esp_wifi_on]");
    }
    else {
      udp_send(String(receivedChars));
    }
  }
  newData = false;
}
