/*
    ENSC405W - TriWave Systems (Group 5)
    Beacon Sketch
 */



#include <WiFi.h>
#include <WiFiUdp.h>

const char * ssid = "myhostap";
const char * password = "1234567890";
const char * hostAddress = "192.168.1.73";
const int UdpPort = 3333;

char incomingPacket[255];
bool system_on = false;
String packet;

WiFiUDP Udp;

void setup()
{
  Serial.begin(115200);
  Serial.printf("Connecting to WiFi network: ", ssid);
  WiFi.begin(ssid, password);
  while (WiFi.status() != WL_CONNECTED) {delay(500); Serial.print(".");}
  Serial.println("WiFi Connected!");
  Udp.begin(UdpPort);
  Serial.printf("Local IP %s, Local UDP port %d\n", WiFi.localIP().toString().c_str(), UdpPort);
  Serial.print("Gatway IP: ");
  Serial.println(WiFi.gatewayIP());
}


void loop()
{
  int packetSize = Udp.parsePacket();
  if (packetSize)
  {
    Serial.printf("Received %d bytes from %s, port %d\n", packetSize, Udp.remoteIP().toString().c_str(), Udp.remotePort());
    int len = Udp.read(incomingPacket, 255);
    packet = String(incomingPacket);
    if (len > 0){incomingPacket[len] = 0;}
    Serial.printf("UDP packet contents: %s\n", incomingPacket);

    if (packet.indexOf("start") >= 0){
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
  }

  
  if (system_on) {

  	//TODO add ranging


    Udp.beginPacket(hostAddress, UdpPort);
    Udp.printf("BEACON_MAC|TAG_MAC|%u \n", millis()/1000); // TEMP data
    Udp.endPacket();
  }



  delay(1000);
}
