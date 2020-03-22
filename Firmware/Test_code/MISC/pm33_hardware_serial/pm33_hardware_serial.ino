
const byte numChars = 50;
char receivedChars[numChars];
boolean newData = false;
bool system_on = false;

void setup() {
  Serial.begin(9600);
  Serial.println("<Arduino>");
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

  recvWithStartEndMarkers();
  if (newData == true) {
    Serial.println(String(receivedChars));
    if (String(receivedChars).indexOf("start") >= 0) {
      Serial.println("<start_ack>"); system_on = true;
    }
    else if (String(receivedChars).indexOf("end") >= 0) {
      Serial.println("<end_ack>"); system_on = false;
    }
    newData = false;
  }

  if (system_on) {
    Serial.println("<data>");
   
  }
  

}
