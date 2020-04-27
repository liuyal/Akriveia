/*
    ENSC405W - TriWave Systems (Group 5)
    Beacon WiFi Test Sketch
*/
#include <WiFi.h>
#include <WiFiUdp.h>

const char * ssid = "akriveia";
const char * password = "1234567890";
const char * hostAddress = "10.0.0.1";
const int UdpPort = 3333;

char incomingPacket[255];
bool system_on = false;
int wifi_timeout = 10 * 1000;
String packet;

WiFiUDP Udp;

void setup()
{
  Serial.begin(115200);
  Serial.printf("Connecting to WiFi network: ", ssid);
  WiFi.begin(ssid, password);
  unsigned long start_wait = millis();
  while (WiFi.status() != WL_CONNECTED && millis() - start_wait <= wifi_timeout) {
    Serial.print(".");
    delay(500);
  }
  if (WiFi.status() == WL_CONNECTED) {
    Serial.println("\nWiFi Connected!");
    Udp.begin(UdpPort);
    Serial.printf("Local IP %s, Local UDP port %d\n", WiFi.localIP().toString().c_str(), UdpPort);
    Serial.print("Gatway IP: ");
    Serial.println(WiFi.gatewayIP());
  }
  else {
    Serial.println("\nWiFi Not Connected...");
  }
}


void loop()
{
  String ranging_info = ("BEACON_MAC|TAG_MAC|%u");

  if (Serial.available() > 0) {
    packet = Serial.readString();
    if (packet.indexOf("start") >= 0) {
      Serial.println("start_ack"); system_on = true;
    }
    else if (packet.indexOf("end") >= 0) {
      Serial.println("end_ack"); system_on = false;
    }
  }

  if (WiFi.status() == WL_CONNECTED) {
    int packetSize = Udp.parsePacket();
    if (packetSize) {
      Serial.printf("Received %d bytes from %s, port %d\n", packetSize, Udp.remoteIP().toString().c_str(), Udp.remotePort());
      int len = Udp.read(incomingPacket, 255);
      packet = String(incomingPacket);
      if (len > 0)incomingPacket[len] = 0;
      Serial.printf("UDP packet contents: %s", incomingPacket);

      if (packet.indexOf("start") >= 0) {
        Serial.println("Send: start_ack");
        system_on = true;
        Udp.beginPacket(hostAddress, UdpPort);
        Udp.printf("start_ack\n");
        Udp.endPacket();
      }
      else if (packet.indexOf("end") >= 0) {
        Serial.println("Send: end_ack");
        system_on = false;
        Udp.beginPacket(hostAddress, UdpPort);
        Udp.printf("end_ack\n");
        Udp.endPacket();
      }
      else if (packet.indexOf("ping") >= 0) {
        Serial.println("Recieved Server Ping!");
        Udp.beginPacket(hostAddress, UdpPort);
        Udp.printf("ping_ack\n");
        Udp.printf("device_info\n");// TODO add device info
        Udp.endPacket();
      }
      else if (packet.indexOf("info") >= 0) {
        Serial.println("Send: device info");
        Udp.beginPacket(hostAddress, UdpPort);
        Udp.printf("device_info\n");// TODO add device info
        Udp.endPacket();
      }
    }
  }


  if (system_on) {
    Serial.println(ranging_info.c_str());

    if (WiFi.status() == WL_CONNECTED) {
      Udp.beginPacket(hostAddress, UdpPort);
      Udp.printf(ranging_info.c_str());
      Udp.endPacket();
    }
  }

  delay(500);
}
