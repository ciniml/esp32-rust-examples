#include <stdio.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_system.h"

extern void rust_main(void);

void app_main()
{
    rust_main();
    fflush(stdout);
    esp_restart();
}
