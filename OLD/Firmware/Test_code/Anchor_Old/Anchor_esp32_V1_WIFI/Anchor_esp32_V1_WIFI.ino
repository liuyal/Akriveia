#include <DW1000Ng.hpp>
#include <DW1000NgUtils.hpp>
#include <DW1000NgRanging.hpp>
#include <DW1000NgRTLS.hpp>
#include <WiFi.h>
#include <WiFiUdp.h>

#if defined(ESP32)
const uint8_t PIN_SCK = 18;
const uint8_t PIN_MOSI = 23;
const uint8_t PIN_MISO = 19;
const uint8_t PIN_SS = 2;
const uint8_t PIN_RST = 15;
const uint8_t PIN_IRQ = 17;
#else
const uint8_t PIN_RST = 9; // reset pin
const uint8_t PIN_IRQ = 2; // irq pin
const uint8_t PIN_SS = SS; // spi select pin
#endif

char* EUI = "AA:BB:CC:DD:EE:FF:00:01";
uint16_t dex = 1;
bool is_head = true;

uint16_t netID;
uint16_t next_anchor;

double range_self;
uint16_t blink_rate = 200;
byte tag_shortAddress[] = {0x05, 0x00};
String TAG_EUI = "AA:BB";

const char * ssid = "AP";
const char * password = "1234567890";
const char * hostAddress = "10.0.0.1";
const int UdpPort = 3333;

char incomingPacket[255];
bool system_on = false;
bool wifi_on = false;
int wifi_timeout = 10 * 1000;
String packet;

WiFiUDP Udp;

device_configuration_t DEFAULT_CONFIG = {
  false,
  true,
  true,
  true,
  false,
  SFDMode::STANDARD_SFD,
  Channel::CHANNEL_5,
  DataRate::RATE_850KBPS,
  PulseFrequency::FREQ_16MHZ,
  PreambleLength::LEN_256,
  PreambleCode::CODE_3
};

frame_filtering_configuration_t ANCHOR_FRAME_FILTER_CONFIG = {
  false,
  false,
  true,
  false,
  false,
  false,
  false,
  true /* This allows blink frames */
};

void setup() {
  Serial.begin(115200);
  delay(500);
  Serial.printf("Connecting to WiFi network: ", ssid);
//  WiFi.begin(ssid, password);
  unsigned long start_wait = millis();
  while (WiFi.status() != WL_CONNECTED && millis() - start_wait <= wifi_timeout) {
    Serial.print("."); delay(500);
  }
  if (WiFi.status() == WL_CONNECTED) {
    Serial.println("\nWiFi Connected!");
    Udp.begin(UdpPort);
    Serial.printf("Local IP %s, Local UDP port %d\n", WiFi.localIP().toString().c_str(), UdpPort);
    Serial.print("Gatway IP: ");
    Serial.println(WiFi.gatewayIP());
    wifi_on = true;
  }
  else {
    Serial.println("\nWiFi Not Connected...");
    wifi_on = false;
  }
  delay(10);

  if (!is_head) ANCHOR_FRAME_FILTER_CONFIG.allowReservedFive = false;
  netID = dex;
  next_anchor = dex + 1;
  Serial.print(F("### DW1000Ng-arduino-ranging-anchor-")); Serial.print(dex); Serial.println(" ###");
  DW1000Ng::initializeNoInterrupt(PIN_SS, PIN_RST);
  Serial.println(F("DW1000Ng initialized ..."));
  DW1000Ng::applyConfiguration(DEFAULT_CONFIG);
  DW1000Ng::enableFrameFiltering(ANCHOR_FRAME_FILTER_CONFIG);
  DW1000Ng::setEUI(EUI);
  DW1000Ng::setPreambleDetectionTimeout(64);
  DW1000Ng::setSfdDetectionTimeout(273);
  DW1000Ng::setReceiveFrameWaitTimeoutPeriod(5000);
  DW1000Ng::setNetworkId(RTLS_APP_ID);
  DW1000Ng::setDeviceAddress(netID);
  DW1000Ng::setAntennaDelay(16436);
  delay(10);
  Serial.println(F("Committed configuration ..."));
  char msg[128];
  DW1000Ng::getPrintableDeviceIdentifier(msg);
  Serial.print("Device ID: "); Serial.println(msg);
  DW1000Ng::getPrintableExtendedUniqueIdentifier(msg);
  Serial.print("Unique ID: "); Serial.println(msg);
  DW1000Ng::getPrintableNetworkIdAndShortAddress(msg);
  Serial.print("Network ID & Device Address: "); Serial.println(msg);
  DW1000Ng::getPrintableDeviceMode(msg);
  Serial.print("Device mode: "); Serial.println(msg);
  delay(10);
}

void loop() {
  
  if (system_on) {
    String ranging_info;
    RangeAcceptResult result;

    if (is_head) {
      if (DW1000NgRTLS::receiveFrame()) {
        size_t recv_len = DW1000Ng::getReceivedDataLength();
        byte recv_data[recv_len];
        DW1000Ng::getReceivedData(recv_data, recv_len);
        if (recv_data[0] == BLINK) {
          DW1000NgRTLS::transmitRangingInitiation(&recv_data[2], tag_shortAddress);
          DW1000NgRTLS::waitForTransmission();
          result = DW1000NgRTLS::anchorRangeAccept(NextActivity::RANGING_CONFIRM, next_anchor);
          if (result.success) {
            ranging_info = String(EUI) + '|' + String(TAG_EUI) + '|' + String(result.range);
            Serial.println(ranging_info);
          }
        }
      }
    }
    else {
      result = DW1000NgRTLS::anchorRangeAccept(NextActivity::RANGING_CONFIRM, next_anchor);
      if (result.success) {
        delay(2);
        ranging_info = String(EUI) + '|' + String(TAG_EUI) + '|' + String(result.range);
        Serial.println(ranging_info);
      }
    }

    if (wifi_on) {
      Udp.beginPacket(hostAddress, UdpPort);
      Udp.printf(ranging_info.c_str());
      Udp.endPacket();
    }
  }
  

  if (Serial.available() > 0) {
    packet = Serial.readString();
    if (packet.indexOf("start") >= 0) {
      Serial.println("start_ack"); system_on = true;
    }
    else if (packet.indexOf("end") >= 0) {
      Serial.println("end_ack"); system_on = false;
    }
  }

  if (wifi_on) {
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
}
