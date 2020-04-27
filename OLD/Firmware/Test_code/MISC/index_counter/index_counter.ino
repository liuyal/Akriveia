
int n_tag = 2;
int counter = 0;
int index_ = 0;
int count_per_index = 10;
int max_count = count_per_index  * n_tag - 1;

byte A[] = {B00000000, B00000000, B11111111, B11101110, B11011101, B11001100, B10111011, B10101010};
byte B[] = {B00000001, B00000000, B11111111, B11101110, B11011101, B11001100, B10111011, B10101010};
byte C[] = {B00000010, B00000000, B11111111, B11101110, B11011101, B11001100, B10111011, B10101010};

void setup() {
  Serial.begin(9600);
}


void loop() {
  if (counter == 0) index_ = 0;
  else index_ = counter / count_per_index % counter ;
  eui[0] = highByte(index_) << 4 | lowByte(index_);
  

  Serial.println(String(counter) + "|" + String(index_) + "|" + eui[0]);


  if (counter >= max_count) counter = 0;
  else counter++;

  delay(500);
}
