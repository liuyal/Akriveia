/*
    ENSC405W - TriWave Systems (Group 5)
    Beacon (Advertise and Scan) Sketch
 */

#include <BLEDevice.h>
#include <BLEUtils.h>
#include <BLEAdvertisedDevice.h>
#include <BLEScan.h>
#include <BLEServer.h>

#define SERVICE_UUID        "5b900c95-589f-4b29-8577-266f3d3ad2f8"
#define CHARACTERISTIC_UUID "81340f69-e519-4847-a3a0-7cfb26b9d88a"

bool system_on = false;
String input = "";
int scanTime = 1;

char *mac_list[] = {"cc:50:e3:9c:4c:c2",
                    "cc:50:e3:9c:48:86"};

class MyAdvertisedDeviceCallbacks : public BLEAdvertisedDeviceCallbacks {
  public:

  void onResult(BLEAdvertisedDevice advertisedDevice) {
    for (int i = 0; i < sizeof(mac_list)/sizeof(mac_list[0]); i++){
      if (advertisedDevice.getAddress().toString()==mac_list[i]){
        Serial.print(advertisedDevice.getName().c_str());
        Serial.print("|");
        Serial.print(advertisedDevice.getAddress().toString().c_str());
        Serial.print("|");
        Serial.print(advertisedDevice.getRSSI());
        Serial.println("");
      }
    }
  }
};

// Advertising
void setup() {
  Serial.begin(115200);
  BLEDevice::init("Beacon_A");
  
  BLEServer *pServer = BLEDevice::createServer();
  BLEService *pService = pServer->createService(SERVICE_UUID);
  BLECharacteristic *pCharacteristic = pService->createCharacteristic(CHARACTERISTIC_UUID, BLECharacteristic::PROPERTY_READ | BLECharacteristic::PROPERTY_WRITE);
  pCharacteristic->setValue("AAA");
  pService->start();

  BLEAdvertising *pAdvertising = BLEDevice::getAdvertising();
  pAdvertising->addServiceUUID(SERVICE_UUID);
  pAdvertising->setScanResponse(true);
  pAdvertising->setMinPreferred(0x12);
  BLEDevice::startAdvertising();
  
}

// Scanning
void loop() {

  if (Serial.available() > 0) {
    input = Serial.readString();
    if (input.indexOf("start") >= 0) { Serial.println("ack"); system_on = true; }
    else if (input.indexOf("end") >= 0) { Serial.println("ack"); system_on = false; }
  }

  if (system_on) {
    BLEScan* beacon = BLEDevice::getScan();
    MyAdvertisedDeviceCallbacks cb;
    beacon->setAdvertisedDeviceCallbacks(&cb);
    beacon->setActiveScan(true);
    BLEScanResults foundClients = beacon->start(scanTime);
  }
}
