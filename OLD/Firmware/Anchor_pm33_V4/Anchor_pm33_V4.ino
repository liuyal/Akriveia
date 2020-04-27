#include <EEPROM.h>
#include <DW1000Ng.hpp>
#include <DW1000NgUtils.hpp>
#include <DW1000NgRanging.hpp>
#include <DW1000NgRTLS.hpp>

const uint8_t PIN_RST = 9; // reset pin
const uint8_t PIN_IRQ = 2; // irq pin
const uint8_t PIN_SS = SS; // spi select pin

char* EUI = "AA:BB:CC:DD:EE:FF:00:0A";
uint16_t netID = 10;
bool is_head = true;
uint16_t next_anchor;
byte beacon_list[] = {0x0A, 0x0B, 0x0C};
int next_index = 0;

int blink_rate = 200;
int wait_timeout = 600;
int head_wait_timeout = 200;
byte tag_shortAddress[] = {0x00, 0x00};

const byte numChars = 50;
char receivedChars[numChars];
boolean newData = false;

bool system_on = false;
int eeprom_address = 0;
byte system_state;

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

  system_state = EEPROM.read(eeprom_address);
  if (system_state == 0x01) system_on = true;
  else system_on = false;

  Serial.println("<[" + String(EUI) + "|pm33_on]>");
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

void recvWithStartEndMarkers() {
  static boolean recvInProgress = false;
  static byte ndx = 0;
  char startMarker = '<';
  char endMarker = '>';
  char rc;
  while (Serial.available() > 0 && newData == false) {
    rc = Serial.read();
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
        double range = result.range;
        if ( range <= 0.01) range = 0.01;
        else if (range > 300) range = 300.00;
        ranging_info = "<[" + String(EUI) + "|range_ack|0x00";
        ranging_info += String(highByte(recv_data[2]), HEX) + String(lowByte(recv_data[2]), HEX);
        ranging_info += "|" + String(range) + "]>";
        Serial.println(ranging_info);
      }
    }
  }
}

void transmit() {
  byte rangingReport[] = {DATA, SHORT_SRC_AND_DEST, DW1000NgRTLS::increaseSequenceNumber(), 0, 0, 0, 0, 0, 0, 0x60, 0, 0 };
  byte next_beacon_address[] = {beacon_list[next_index], 0x00};
  double range = 88;
  DW1000Ng::getNetworkId(&rangingReport[3]);
  memcpy(&rangingReport[5], next_beacon_address, 2);
  DW1000Ng::getDeviceAddress(&rangingReport[7]);
  DW1000NgUtils::writeValueToBytes(&rangingReport[10], static_cast<uint16_t>((range * 1000)), 2);
  DW1000Ng::setTransmitData(rangingReport, sizeof(rangingReport));
  DW1000Ng::startTransmit();
}

void wait() {
  int counter = 0;
  int time_out = head_wait_timeout;
  if (!is_head) time_out = wait_timeout;
  while (counter < time_out) {
    if (DW1000NgRTLS::receiveFrame()) {
      size_t recv_len = DW1000Ng::getReceivedDataLength();
      byte recv_data[recv_len];
      DW1000Ng::getReceivedData(recv_data, recv_len);
      if (recv_data[9] == 0x60) break;
    }
    counter++;
    cmd_event();
  }
}

void cmd_event() {
  recvWithStartEndMarkers();
  if (newData == true) {
    Serial.println(String(receivedChars));
    if (String(receivedChars).indexOf("start") >= 0) {
      Serial.println("<[" + String(EUI) + "|start_ack]>");
      system_on = true;
      EEPROM.write(eeprom_address, 0x01);
    }
    else if (String(receivedChars).indexOf("end") >= 0) {
      Serial.println("<[" + String(EUI) + "|end_ack]>");
      system_on = false;
      EEPROM.write(eeprom_address, 0x00);
    }
    else if (String(receivedChars).indexOf("ping") >= 0) {
      Serial.println("<[" + String(EUI) + "|ping_ack]>");
    }
    else if (String(receivedChars).indexOf("get_eui") >= 0) {
      Serial.println("<" + String(EUI) + "|eui_ack>");
    }
    newData = false;
  }
}

void loop() {

  if (system_on) {
    for (int j = 0; j < 100; j++) range();
    for (int j = 0; j < 20; j++) transmit();
    wait();
  }

  cmd_event();
}
