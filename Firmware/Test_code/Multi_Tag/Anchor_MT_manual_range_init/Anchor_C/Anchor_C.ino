#include <DW1000Ng.hpp>
#include <DW1000NgUtils.hpp>
#include <DW1000NgRanging.hpp>
#include <DW1000NgRTLS.hpp>

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

char* EUI = "AA:BB:CC:DD:EE:FF:00:0C";
uint16_t dex = 12;
bool is_head = false;
bool is_tail = false;

uint16_t netID;
uint16_t next_anchor;
byte beacon_list[] = {0x0A, 0x0B, 0x0C};

double range[sizeof(beacon_list)] = {};

int blink_rate = 100;
byte tag_shortAddress[] = {0x00, 0x00};
byte eui[] = {B00000000, B00000000, B11111111, B11101110, B11011101, B11001100, B10111011, B10101010};

int n_tag = 2;
int counter = 0;
int index_ = 0;
int count_per_index = 50;
int max_count = count_per_index  * n_tag - 1;

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

void transmitRangeReport() {
  byte rangingReport[] = {DATA, SHORT_SRC_AND_DEST, DW1000NgRTLS::increaseSequenceNumber(), 0, 0, 0, 0, 0, 0, 0x60, 0, 0 };
  byte main_anchor_address[] = {beacon_list[0], 0x00};
  DW1000Ng::getNetworkId(&rangingReport[3]);
  memcpy(&rangingReport[5], main_anchor_address, 2);
  DW1000Ng::getDeviceAddress(&rangingReport[7]);
  DW1000NgUtils::writeValueToBytes(&rangingReport[10], static_cast<uint16_t>((range[0] * 1000)), 2);
  DW1000Ng::setTransmitData(rangingReport, sizeof(rangingReport));
  DW1000Ng::startTransmit();
}

void loop() {
  String ranging_info;
  RangeAcceptResult result;
  result = DW1000NgRTLS::anchorRangeAccept(NextActivity::RANGING_CONFIRM, next_anchor);
  if (result.success) {
    delay(2);
    range[0] = result.range;
    transmitRangeReport();
    ranging_info = "<0x00|" + String(dex, HEX) + "|" + String(result.range) + '>';
    Serial.println(ranging_info);
  }

}
