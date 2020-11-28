

use usbd_hid::descriptor::generator_prelude::*;


#[gen_hid_descriptor(
(collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = KEYBOARD) = {
    (usage_page = KEYBOARD, usage_min = 0xE0, usage_max = 0xE7) = {
        #[packed_bits 8] #[item_settings data,variable,absolute] modifier=input;
    };
    (usage_page = LEDS, usage_min = 0x01, usage_max = 0x05) = {
        #[packed_bits 5] #[item_settings data,variable,absolute] leds=output;
    };
    (usage_page = KEYBOARD, usage_min = 0x00, usage_max = 0x65) = {
        #[item_settings data,array,absolute] keycodes=input;
    };

    (collection = PHYSICAL, usage = POINTER) = {
        (usage_page = BUTTON, usage_min = BUTTON_1, usage_max = BUTTON_3) = {
            #[packed_bits 3] #[item_settings data,variable,absolute] buttons=input;
        };
        (usage_page = GENERIC_DESKTOP,) = {
            (usage = X,) = {
                #[item_settings data,variable,relative] x=input;
            };
            (usage = Y,) = {
                #[item_settings data,variable,relative] y=input;
            };
        };
    };
},
)]
pub struct KeyboardReport2 {
    pub modifier: u8,
    pub leds: u8,
    pub keycodes: [u8; 6],
    pub buttons: u8,
    pub x: i8,
    pub y: i8,
}