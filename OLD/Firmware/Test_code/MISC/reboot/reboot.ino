#include <avr/wdt.h>

const byte numChars = 50;
char receivedChars[numChars];
boolean newData = false;


void softwareReset(uint8_t prescaller) {
  wdt_enable(prescaller);
  while (1) {}
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

void CMD_EVENT() {
  recvWithStartEndMarkers();
  if (newData == true) {
    Serial.println(String(receivedChars));
    if (String(receivedChars).indexOf("start") >= 0) {
      Serial.println("<[start_ack]>");
    }
    else if (String(receivedChars).indexOf("end") >= 0) {
      Serial.println("<[end_ack]>");
    }
    else if (String(receivedChars).indexOf("reboot") >= 0) {
      Serial.println("<[pm33_reboot_ack]>");
      softwareReset(WDTO_60MS);
    }
    newData = false;
  }
}


void setup() {
  // put your setup code here, to run once:
  Serial.begin(9600);
  Serial.println("Testing software reboot");
}

void loop() {
  // put your main code here, to run repeatedly:
  CMD_EVENT();
}
