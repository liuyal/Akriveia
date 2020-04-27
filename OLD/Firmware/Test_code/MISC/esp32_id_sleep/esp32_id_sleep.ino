/*
   ENSC405W - TriWare Systems (Group 5)
   ID tag sketch
*/

#include <BLEDevice.h>
#include <BLEUtils.h>
#include <BLEServer.h>

// See the following for generating UUIDs:
// https://www.uuidgenerator.net/

#define SERVICE_UUID        "4fafc201-1fb5-459e-8fcc-c5c9c331914b"
#define CHARACTERISTIC_UUID "beb5483e-36e1-4688-b7f5-ea07361b26a8"
#define THRESHOLD 3

RTC_DATA_ATTR int bootCount = 0;
touch_pad_t touchPin;
volatile bool timer_on = false;


void print_wakeup_reason() {
  esp_sleep_wakeup_cause_t wakeup_reason;
  wakeup_reason = esp_sleep_get_wakeup_cause();
  switch (wakeup_reason)
  {
    case ESP_SLEEP_WAKEUP_TOUCHPAD : Serial.println("Wakeup caused by touchpad"); break;
    default : Serial.printf("Wakeup was not caused by deep sleep: %d\n", wakeup_reason); break;
  }
}

void print_wakeup_touchpad() {
  touch_pad_t pin;

  touchPin = esp_sleep_get_touchpad_wakeup_status();

  switch (touchPin)
  {
    case 0  : Serial.println("Touch detected on GPIO 15"); break;
    default : Serial.println("Wakeup not by touchpad"); break;
  }
}

void callback() {
  Serial.println("Touch ISR called...");
  timer_on = true;
}


void setup() {
  Serial.begin(115200);
  BLEDevice::init("ID Tag");
  BLEServer *pServer = BLEDevice::createServer();
  BLEService *pService = pServer->createService(SERVICE_UUID);
  BLECharacteristic *pCharacteristic = pService->createCharacteristic(
                                         CHARACTERISTIC_UUID,
                                         BLECharacteristic::PROPERTY_READ |
                                         BLECharacteristic::PROPERTY_WRITE
                                       );

  pCharacteristic->setValue("Hello World");
  pService->start();


  ++bootCount;
  Serial.println("Boot number: " + String(bootCount));

  print_wakeup_reason();
  print_wakeup_touchpad();

  //Setup interrupt on Touch Pad 3 (GPIO15)
  touchAttachInterrupt(T3, callback, THRESHOLD);


  //Configure Touchpad as wakeup source
  esp_sleep_enable_touchpad_wakeup();

  delay(2000); // delay for touch ISR to engage

  if (timer_on) {
    BLEAdvertising *pAdvertising = BLEDevice::getAdvertising();
    pAdvertising->addServiceUUID(SERVICE_UUID);
    pAdvertising->setScanResponse(true);
    //pAdvertising->setMinPreferred(0x06);  // functions that help with iPhone connections issue
    pAdvertising->setMinPreferred(0x12);
    BLEDevice::startAdvertising();
    Serial.println("ID tag advertising for 10 seconds... ");
    delay(10000);
    Serial.println("Going back to deep sleep mode...");
    esp_deep_sleep_start();
  }
  else {
    Serial.println("Going to sleep now");
    timer_on = false;
    esp_deep_sleep_start();
    Serial.println("This will never be printed"); // debugging
  }

}

void loop() {
}
