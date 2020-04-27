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

char* EUI = "AA:BB:CC:DD:EE:FF:00:0A";
uint16_t dex = 10;
bool is_head = true;
bool is_tail = false;

uint16_t netID;
uint16_t next_anchor;

double range_self;
byte anchors[] = {0x0A, 0x00, 0x0B, 0x00, 0x0C, 0x00};
uint16_t blink_rate = 200;
byte tag_shortAddress[] = {0x00, 0x00};
String TAG_EUI = "AA:BB:CC:DD:EE:FF:00:00";

const byte numChars = 50;
char receivedChars[numChars];
boolean newData = false;
bool system_on = false;

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

String byte_2_hex(byte data[], size_t n) {
  size_t len = 16;
  char buffer[len];
  for (unsigned int i = 9; i >= 2; i--) {
    byte nib1 = (data[11 - i] >> 4) & 0x0F;
    byte nib2 = (data[11 - i] >> 0) & 0x0F;
    buffer[(i - 2) * 2 + 0] = nib1  < 0xA ? '0' + nib1  : 'A' + nib1  - 0xA;
    buffer[(i - 2) * 2 + 1] = nib2  < 0xA ? '0' + nib2  : 'A' + nib2  - 0xA;
  }
  buffer[len * 2] = '\0';
  String id0 = String(buffer[0]) + String(buffer[1]) + ':' + String(buffer[2]) + String(buffer[3]) + ':';
  String id1 = String(buffer[4]) + String(buffer[5]) + ':' + String(buffer[6]) + String(buffer[7]) + ':';
  String id2 = String(buffer[8]) + String(buffer[9]) + ':' + String(buffer[10]) + String(buffer[11]) + ':';
  String id3 = String(buffer[12]) + String(buffer[13]) + ':' + String(buffer[14]) + String(buffer[15]);
  return id0 + id1 + id2 + id3;
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
          //  DW1000NgRTLS::waitForTransmission();
          result = DW1000NgRTLS::anchorRangeAccept(NextActivity::RANGING_CONFIRM, next_anchor);
          if (result.success) {
            size_t len = recv_len;
            byte data[len] = {};
            memcpy(data, recv_data, len);
            TAG_EUI = byte_2_hex(data, len);
            ranging_info = '<' + String(EUI) + '|' + TAG_EUI + '|' + String(result.range) + '>';
            Serial.println(ranging_info);
          }
        }
      }
    }
    else if (!is_head && is_tail) {
      result = DW1000NgRTLS::anchorRangeAccept(NextActivity::ACTIVITY_FINISHED, blink_rate);
      if (result.success) {
        delay(2);
        ranging_info = '<' + String(EUI) + '|' + String(TAG_EUI) + '|' + String(result.range) + '>';
        Serial.println(ranging_info);
      }
    }
    else {
      result = DW1000NgRTLS::anchorRangeAccept(NextActivity::RANGING_CONFIRM, next_anchor);
      if (result.success) {
        delay(1);
        ranging_info = '<' + String(EUI) + '|' + String(TAG_EUI) + '|' + String(result.range) + '>';
        Serial.println(ranging_info);
      }
    }
  }

  recvWithStartEndMarkers();
  if (newData == true) {
    Serial.println(String(receivedChars));
    if (String(receivedChars).indexOf("start") >= 0) {
      Serial.println("<start_ack>");
      system_on = true;
    }
    else if (String(receivedChars).indexOf("end") >= 0) {
      Serial.println("<end_ack>");
      system_on = false;
    }
    else if (String(receivedChars).indexOf("ping") >= 0) {
      Serial.print("<ping_ack");
      Serial.print('|' + String(EUI) + '|');
      Serial.println('>');
    }
    newData = false;
  }
}

