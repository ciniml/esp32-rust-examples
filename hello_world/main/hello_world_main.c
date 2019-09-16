#include <stdio.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_system.h"
#include "esp_spi_flash.h"

extern void rust_main(void);

void app_main()
{
    printf("Hello world!\n");

    rust_main();

    vTaskDelay(pdMS_TO_TICKS(1000));

    fflush(stdout);
    esp_restart();
}
