#![no_main]
#![no_std]
// #![allow(warnings, unused)]

mod buttons;
mod delay;
mod strings;
mod terminal;

use panic_halt as _;

#[rtic::app(device = wio_terminal::pac, peripherals = true, monotonic = rtic::cyccnt::CYCCNT,
    dispatchers = [SDHC0, SDHC1, DAC_OTHER, DAC_EMPTY_0, DAC_EMPTY_1, DAC_RESRDY_0, DAC_RESRDY_1])]
mod app {
    use wio::prelude::*;
    use wio_terminal as wio;

    // Time
    use rtic::cyccnt::{Instant, U32Ext};
    use wio::hal::clock::GenericClockController;

    // IO
    use wio::hal::gpio::*;
    use wio::{Pins, Sets};

    // USB
    use usb_device::bus::UsbBusAllocator;
    use usb_device::prelude::*;
    use usbd_hid::descriptor::generator_prelude::*;
    use usbd_hid::descriptor::{KeyboardReport, MouseReport};
    use usbd_hid::hid_class::HIDClass;
    use wio::hal::usb::UsbBus;

    // crate
    use crate::delay::InstDelay;
    use crate::strings::str_to_fixed as stf;
    use crate::terminal::Terminal;

    // Buttons
    use wio::{Button, ButtonEvent};
    use wio_terminal::ButtonController;

    use arrayvec::ArrayString;
    use core::fmt::Write;

    #[resources]
    struct Resources {
        #[task_local]
        user_led: Pa15<Output<OpenDrain>>,

        // USB
        #[task_local]
        usb_bus: UsbDevice<'static, UsbBus>,
        usb_hid: HIDClass<'static, 'static, UsbBus>,

        // Display
        #[task_local]
        terminal: Terminal,
        #[task_local]
        backlight: Pc5<Output<PushPull>>,
        #[task_local]
        #[init(true)]
        backlight_state: bool,

        // Buttons
        button_ctr: ButtonController,
    }

    const PERIOD: u32 = 16_000_000;
    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let mut core = cx.core;
        core.DWT.enable_cycle_counter();

        let mut device = cx.device;

        let mut clocks = GenericClockController::with_external_32kosc(
            device.GCLK,
            &mut device.MCLK,
            &mut device.OSC32KCTRL,
            &mut device.OSCCTRL,
            &mut device.NVMCTRL,
        );

        // PORT
        let pins = Pins::new(device.PORT);
        let mut sets: Sets = pins.split();

        // Blue Led
        let mut user_led = sets.user_led.into_open_drain_output(&mut sets.port);

        // LCD

        // Initialize the ILI9341-based LCD display. Create a black backdrop the size of
        // the screen.
        let (display, backlight) = sets
            .display
            .init(
                &mut clocks,
                device.SERCOM7,
                &mut device.MCLK,
                &mut sets.port,
                58.mhz(),
                &mut InstDelay {},
            )
            .unwrap();
        let mut term = Terminal::new(display);

        term.write_str("Hello! Send text to me over the USB serial port, and I'll display it!");
        term.write_str("\n");
        term.write_str("On linux:\n");
        term.write_str("  sudo stty -F /dev/ttyACM0 115200 raw -echo\n");
        term.write_str("  sudo bash -c \"echo 'Hi' > /dev/ttyACM0\"\n");

        // USB
        static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;

        let bus_allocator = unsafe {
            USB_ALLOCATOR = Some(sets.usb.usb_allocator(
                device.USB,
                &mut clocks,
                &mut device.MCLK,
                &mut sets.port,
            ));
            USB_ALLOCATOR.as_ref().unwrap()
        };

        let usb_hid = HIDClass::new(&bus_allocator, KeyboardReport::desc(), 255);

        let usb_bus = UsbDeviceBuilder::new(&bus_allocator, UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Fake company")
            .product("Serial port")
            .serial_number("TEST")
            .composite_with_iads()
            .build();

        user_led.set_low().unwrap();

        // Buttons
        let button_ctr =
            sets.buttons
                .init(device.EIC, &mut clocks, &mut device.MCLK, &mut sets.port);

        // Start Tasks
        blinky::schedule(cx.start + PERIOD.cycles()).unwrap();
        pwm::spawn().unwrap();

        init::LateResources {
            user_led,
            usb_bus,
            usb_hid,
            terminal: term,
            backlight,
            button_ctr,
        }
    }

    #[task(binds = USB_OTHER)]
    fn usb_other(_: usb_other::Context) {
        usb::spawn().ok();
    }
    #[task(binds = USB_SOF_HSOF)]
    fn usb_sof_hsof(_: usb_sof_hsof::Context) {
        usb::spawn().ok();
    }
    #[task(binds = USB_TRCPT0)]
    fn usb_trcpt0(_: usb_trcpt0::Context) {
        usb::spawn().ok();
    }
    #[task(binds = USB_TRCPT1)]
    fn usb_trcpt1(_: usb_trcpt1::Context) {
        usb::spawn().ok();
    }

    #[task(resources = [usb_bus, usb_hid], priority = 10)]
    fn usb(cx: usb::Context) {
        let usb::Resources {
            usb_bus,
            mut usb_hid,
        } = cx.resources;
        usb_hid.lock(|hid| usb_bus.poll(&mut [hid]));
    }

    #[task(resources = [user_led])]
    fn blinky(cx: blinky::Context) {
        cx.resources.user_led.toggle();
        blinky::schedule(cx.scheduled + PERIOD.cycles()).unwrap();
    }

    #[task(resources = [backlight, backlight_state], priority = 9)]
    fn pwm(cx: pwm::Context) {
        const PERIOD: u32 = 120; // 120KHz PWM
        const FACTOR: u32 = 1024;
        const BRIGHTNESS: u32 = 20; // `PERIOD` max
        const ON: u32 = BRIGHTNESS * FACTOR;
        const OFF: u32 = PERIOD * FACTOR - ON;
        let pwm::Resources {
            backlight,
            backlight_state,
        } = cx.resources;

        let cycles = if *backlight_state {
            backlight.set_low().unwrap();
            OFF
        } else {
            backlight.set_high().unwrap();
            ON
        };
        *backlight_state ^= true;

        pwm::schedule(cx.scheduled + cycles.cycles()).unwrap();
    }

    #[task(resources = [terminal])]
    fn print(cx: print::Context, msg: ArrayString<[u8; 32]>) {
        cx.resources.terminal.write_str(&msg[..]);
    }

    #[task(resources = [usb_hid])]
    fn button(cx: button::Context, event: ButtonEvent) {
        let mut usb_hid = cx.resources.usb_hid;

        match event {
            ButtonEvent {
                button: Button::TopLeft,
                down: true,
            } => {
                usb_hid.lock(|hid| {
                    hid.push_input(&KeyboardReport {
                        modifier: 0,
                        leds: 0,
                        keycodes: [0, 0, 0, 0, 0, 4],
                    })
                });
                print::spawn(stf("TopLeft Down\n")).ok();
            }
            ButtonEvent {
                button: Button::TopLeft,
                down: false,
            } => {
                usb_hid.lock(|hid| {
                    hid.push_input(&KeyboardReport {
                        modifier: 0,
                        leds: 0,
                        keycodes: [0, 0, 0, 0, 0, 0],
                    })
                });
                print::spawn(stf("TopLeft Up\n")).ok();
            }
            ButtonEvent {
                button: Button::Right,
                down: true,
            } => {
                usb_hid.lock(|hid| {
                    hid.push_input(&KeyboardReport {
                        modifier: 0,
                        leds: 0,
                        keycodes: [0, 0, 0, 0, 0, 0x4f],
                    })
                });
                print::spawn(stf("Right Up\n")).ok();
            }
            ButtonEvent {
                button: Button::Right,
                down: false,
            } => {
                usb_hid.lock(|hid| {
                    hid.push_input(&KeyboardReport {
                        modifier: 0,
                        leds: 0,
                        keycodes: [0, 0, 0, 0, 0, 0],
                    })
                });
                print::spawn(stf("Right Down\n")).ok();
            }
            ButtonEvent { .. } => {}
        }
    }

    /// task from macro does not currently work using pre-generated
    /// ```
    /// use crate::buttons::prelude::*;
    /// button_interrupt!(button_ctr, button);
    /// ```
    # [ task ( binds = EIC_EXTINT_3 , resources = [ button_ctr ] ) ]
    fn _btn_intr_3(mut cx: _btn_intr_3::Context) {
        if let Some(event) = cx.resources.button_ctr.lock(|ctl| ctl.interrupt_extint3()) {
            button::spawn(event).ok();
        }
    }
    # [ task ( binds = EIC_EXTINT_4 , resources = [ button_ctr ] ) ]
    fn _btn_intr_4(mut cx: _btn_intr_4::Context) {
        if let Some(event) = cx.resources.button_ctr.lock(|ctl| ctl.interrupt_extint4()) {
            button::spawn(event).ok();
        }
    }
    # [ task ( binds = EIC_EXTINT_5 , resources = [ button_ctr ] ) ]
    fn _btn_intr_5(mut cx: _btn_intr_5::Context) {
        if let Some(event) = cx.resources.button_ctr.lock(|ctl| ctl.interrupt_extint5()) {
            button::spawn(event).ok();
        }
    }
    # [ task ( binds = EIC_EXTINT_7 , resources = [ button_ctr ] ) ]
    fn _btn_intr_7(mut cx: _btn_intr_7::Context) {
        if let Some(event) = cx.resources.button_ctr.lock(|ctl| ctl.interrupt_extint7()) {
            button::spawn(event).ok();
        }
    }
    # [ task ( binds = EIC_EXTINT_10 , resources = [ button_ctr ] ) ]
    fn _btn_intr_10(mut cx: _btn_intr_10::Context) {
        if let Some(event) = cx.resources.button_ctr.lock(|ctl| ctl.interrupt_extint10()) {
            button::spawn(event).ok();
        }
    }
    # [ task ( binds = EIC_EXTINT_11 , resources = [ button_ctr ] ) ]
    fn _btn_intr_11(mut cx: _btn_intr_11::Context) {
        if let Some(event) = cx.resources.button_ctr.lock(|ctl| ctl.interrupt_extint11()) {
            button::spawn(event).ok();
        }
    }
    # [ task ( binds = EIC_EXTINT_12 , resources = [ button_ctr ] ) ]
    fn _btn_intr_12(mut cx: _btn_intr_12::Context) {
        if let Some(event) = cx.resources.button_ctr.lock(|ctl| ctl.interrupt_extint12()) {
            button::spawn(event).ok();
        }
    }
}
