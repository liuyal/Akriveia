
#include <DW1000Ng.hpp>
#include <DW1000NgUtils.hpp>
#include <DW1000NgRanging.hpp>
#include <DW1000NgRTLS.hpp>

const uint8_t PIN_RST = 9; // reset pin
const uint8_t PIN_IRQ = 2; // irq pin
const uint8_t PIN_SS = SS; // spi select pin

char* EUI = "AA:BB:CC:DD:EE:FF:00:0A";
uint16_t netID = 11;
uint16_t next_anchor;
byte beacon_list[] = {0x0A, 0x0B, 0x0C};
int next_index = 0;
bool is_head = true;

int blink_rate = 200;
int wait_timeout = 200;
byte tag_shortAddress[] = {0x00, 0x00};

device_configuration_t DEFAULT_CONFIG = {
  false,
  true,
  true,
  true,
  false,
  SFDMode::STANDARD_SFD,
  Channel::CHANNEL_5,
  DataRate::RATE_850KBPS,
  PulseFrequency::FREQ_64MHZ,
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
  Serial.begin(9600);
  Serial.println("<pm33_on>");
  IndexMapper();
  Serial.print(F("### DW1000Ng-arduino-ranging-anchor-")); Serial.print(netID); Serial.println(" ###");
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
}

void IndexMapper() {
  byte netID_b = highByte(netID) << 4 | lowByte(netID);
  for (int i = 0; i < sizeof(beacon_list); i++) {
    if (beacon_list[i] == netID_b) {
      if (i + 1 == sizeof(beacon_list)) next_index = 0;
      else next_index = i;
    }
  }
  next_anchor = netID + 1;
}

void range() {
  String ranging_info;
  RangeAcceptResult result;
  if (DW1000NgRTLS::receiveFrame()) {
    size_t recv_len = DW1000Ng::getReceivedDataLength();
    byte recv_data[recv_len];
    DW1000Ng::getReceivedData(recv_data, recv_len);
    if (recv_data[0] == BLINK) {
      DW1000NgRTLS::transmitRangingInitiation(&recv_data[2], tag_shortAddress);
      result = DW1000NgRTLS::anchorRangeAccept(NextActivity::RANGING_CONFIRM, next_anchor);
      if (result.success) {
        ranging_info = "<|" + String(EUI);
        ranging_info += "|0x" + String(highByte(recv_data[2]),HEX) + String(lowByte(recv_data[2]),HEX) ;
        ranging_info += "|" + String(result.range) + "|>";
        Serial.println(ranging_info);
      }
    }
  }
}

void transmit() {
  byte rangingReport[] = {DATA, SHORT_SRC_AND_DEST, DW1000NgRTLS::increaseSequenceNumber(), 0, 0, 0, 0, 0, 0, 0x60, 0, 0 };
  byte next_beacon_address[] = {beacon_list[next_index], 0x00};
  double range = 888;
  DW1000Ng::getNetworkId(&rangingReport[3]);
  memcpy(&rangingReport[5], next_beacon_address, 2);
  DW1000Ng::getDeviceAddress(&rangingReport[7]);
  DW1000NgUtils::writeValueToBytes(&rangingReport[10], static_cast<uint16_t>((range * 1000)), 2);
  DW1000Ng::setTransmitData(rangingReport, sizeof(rangingReport));
  DW1000Ng::startTransmit();
}


void wait() {
  int counter = 0;
  while (counter < wait_timeout) {
    if (DW1000NgRTLS::receiveFrame()) {
      size_t recv_len = DW1000Ng::getReceivedDataLength();
      byte recv_data[recv_len];
      DW1000Ng::getReceivedData(recv_data, recv_len);
      if (recv_data[9] == 0x60) break;
    }
    counter++;
  }
}

void loop() {

  for (int j = 0; j < 100; j++) range();
  for (int j = 0; j < 20; j++) transmit();
  wait();

}
